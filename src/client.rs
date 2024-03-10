use std::{fmt::Debug, io::Error, net::SocketAddr, sync::Arc};

use rand::{thread_rng, Rng};
use rsa::{BigUint, Pkcs1v15Encrypt, PublicKey, RsaPublicKey};
use serde_json::json;
use sha1::Digest;
use tokio::{
    io::{AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{
        broadcast::{self, Receiver, Sender},
        Mutex,
    },
};
use tracing::{debug, instrument, trace};

use crate::{
    model::packets::{
        clientbound::{EncryptionRequestPacket, StatusResponsePacket},
        serverbound::{HandshakePacket, LoginRequestPacket, StatusRequestPacket},
    },
    mojang,
    packet::{ClientPacket, RawPacket, ServerPacket},
    response::ResponseData,
};

#[derive(Debug)]
pub struct MinecraftClient {
    pub received_packets: Vec<Box<RawPacket>>,
    reader: Option<Arc<Mutex<ReadHalf<TcpStream>>>>,
    writer: Option<Arc<Mutex<WriteHalf<TcpStream>>>>,
    sender: Sender<RawPacket>,
    receiver: Receiver<RawPacket>,
    listener: Option<tokio::task::JoinHandle<()>>,
}

const USERNAME: &str = "test";
const PASSWORD: &str = "test";

impl MinecraftClient {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(10);
        Self {
            received_packets: Vec::new(),
            reader: None,
            writer: None,
            sender: tx,
            receiver: rx,
            listener: None,
        }
    }

    #[instrument]
    pub async fn login(
        &mut self,
        address: SocketAddr,
        access_token: &str,
        selected_profile: &str,
    ) -> Result<(), Error> {
        debug!("Connecting to server: {:?}", address);

        // Open new connection to try and login
        self.create_stream(address).await?;

        self.send_packet(HandshakePacket {
            protocol_version: 765.into(),
            server_address: address.ip().to_string(),
            server_port: address.port(),
            next_state: 2.into(),
        })
        .await?;
        self.send_packet(LoginRequestPacket {
            username: "test".to_string(),
            uuid: 0,
        })
        .await?;
        self.flush().await?;

        let encryption_request: EncryptionRequestPacket = self.receive_packet().await?;
        // Encryption Request

        let (e, n) = match rsa_der::public_key_from_der(&encryption_request.public_key.0) {
            Ok(it) => it,
            Err(err) => panic!("Error getting public key: {}", err),
        };

        let pub_key_res = RsaPublicKey::new(BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e));
        let public_key = match pub_key_res {
            Ok(it) => it,
            Err(err) => return Err(Error::new(std::io::ErrorKind::Other, err)),
        };

        let mut rng = thread_rng();

        // Encryption Response
        let mut shared_key = vec![0u8; 16];
        rng.fill(shared_key.as_mut_slice());

        let mut hasher = sha1::Sha1::new();
        hasher.update(encryption_request.server_id);
        hasher.update(&shared_key);
        hasher.update(&encryption_request.public_key.0);

        let mut hash = hasher.finalize();
        if hash[0] & 0x80 == 0x80 {
            Self::twos_compliment(&mut hash);
        }
        let hash_str = hash
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        mojang::join_server(access_token, selected_profile, &hash_str).await?;

        let shared_secret = public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, &shared_key)
            .unwrap();
        let verify_token = public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, &encryption_request.verify_token)
            .unwrap();

        Ok(())
    }

    #[instrument]
    pub async fn check_status(address: SocketAddr) -> Result<ResponseData, Error> {
        trace!("Checking server status: {:?}", address);

        let mut stream = TcpStream::connect(address).await?;

        // Initiate connection
        trace!("Sending handshake packet");
        let packet = HandshakePacket {
            protocol_version: 765.into(),
            server_address: address.ip().to_string(),
            server_port: address.port(),
            next_state: 1.into(),
        }
        .serialize()?;
        stream.write(&packet.to_bytes()).await?;

        trace!("Sending status request packet");
        let packet = StatusRequestPacket {}.serialize()?;
        stream.write(&packet.to_bytes()).await?;
        stream.flush().await?;

        trace!("Waiting for status response");
        let raw_packet = RawPacket::from_reader(&mut stream).await?;
        let status_packet: StatusResponsePacket = StatusResponsePacket::deserialize(raw_packet)?;
        trace!("Received status response: {:?}", status_packet);

        Ok(status_packet.get_json()?)
    }

    pub async fn listen(&mut self) -> Result<(), Error> {
        if let Some(old_listener) = &mut self.listener {
            old_listener.abort();
        }

        if let Some(reader) = &mut self.reader {
            let reader = reader.clone();
            let tx = self.sender.clone();

            let listener = tokio::spawn(async move {
                loop {
                    let mut reader = reader.lock().await;
                    let packet = RawPacket::from_reader(&mut *reader).await.unwrap();
                    tx.send(packet).unwrap();
                }
            });
            self.listener = Some(listener);
        } else {
            return Err(Error::new(
                std::io::ErrorKind::NotConnected,
                "Not connected to server",
            ));
        }

        Ok(())
    }

    #[instrument]
    pub async fn receive_packet<T: ClientPacket>(&mut self) -> Result<T, std::io::Error> {
        loop {
            if let Ok(raw_packet) = self.receiver.recv().await {
                trace!("Received packet with id: {:?}", raw_packet.packet_id);
                if raw_packet.packet_id == T::PACKET_ID {
                    return T::deserialize(raw_packet);
                }
            }
        }
    }

    #[instrument]
    pub async fn send_packet<T: ServerPacket>(&mut self, packet: T) -> Result<(), Error> {
        trace!("Sending packet with id: {:?}", T::PACKET_ID);
        let raw_packet = packet.serialize()?;
        if let Some(writer) = &mut self.writer {
            let mut writer = writer.lock().await;
            writer.write(&raw_packet.to_bytes()).await?;
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        if let Some(writer) = &mut self.writer {
            let mut writer = writer.lock().await;
            writer.flush().await?;
        }
        Ok(())
    }

    #[instrument]
    pub async fn create_stream(&mut self, addr: SocketAddr) -> Result<(), Error> {
        self.shutdown().await?;

        trace!("Creating new connection to: {:?}", addr);

        let stream = TcpStream::connect(addr).await?;
        let (reader, writer) = tokio::io::split(stream);
        self.reader = Some(Arc::new(Mutex::new(reader)));
        self.writer = Some(Arc::new(Mutex::new(writer)));
        Ok(())
    }

    #[instrument]
    pub async fn shutdown(&mut self) -> Result<(), Error> {
        trace!("Trying to shutdown old connection");

        if let Some(listener) = &mut self.listener {
            listener.abort();
        }
        if let Some(writer) = &mut self.writer {
            let mut writer = writer.lock().await;
            writer.shutdown().await?;
        }

        Ok(())
    }

    // https://github.com/iceiix/stevenarella/blob/ecf829c544ef6f471402ef01c09626272f6d11f9/protocol/src/protocol/mojang.rs#L186
    fn twos_compliment(data: &mut [u8]) {
        let mut carry = true;
        for i in (0..data.len()).rev() {
            data[i] = !data[i];
            if carry {
                carry = data[i] == 0xFF;
                data[i] = data[i].wrapping_add(1);
            }
        }
    }
}
