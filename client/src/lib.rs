mod async_read_write;
pub mod stream;
mod try_clone;

use std::{
    io::Read,
    net::SocketAddr,
    sync::{atomic::AtomicU8, Arc},
};

use async_read_write::{AsyncPacketReadable, AsyncPacketWritable};
use packet::{
    raw::RawPacket, read::PacketReadable, registry::get_packet, types::VarInt,
    write::PacketWritable, ConnectionState, Packet, PacketDirection,
};
use protocol::clientbound::{DisconnectPacket, StatusResponseJson};
use stream::CountingStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info, instrument, warn, Instrument};
use try_clone::TryClone;

#[derive(Debug)]
pub struct MinecraftClient {
    authentication: Authentication,
}

impl MinecraftClient {
    pub fn authenticated(username: &str, password: &str) -> Self {
        unimplemented!()
    }

    pub fn unauthenticated() -> Self {
        Self {
            authentication: Authentication::Unauthenticated,
        }
    }

    #[instrument]
    pub async fn connect(
        &mut self,
        addr: SocketAddr,
        protocol_version: i32,
    ) -> std::io::Result<Connection> {
        let stream = tokio::net::TcpStream::connect(addr).await?;
        let connection: Connection = Connection::new(stream, protocol_version);
        Ok(connection)
    }
}

#[derive(Debug)]
pub enum Authentication {
    Authenticated(String, String),
    Unauthenticated,
}

#[derive(Debug)]
pub struct Connection {
    protocol_version: i32,
    address: SocketAddr,
    state: Arc<SharedState>,
    read_task: Arc<tokio::task::JoinHandle<()>>,
    write_task: Arc<tokio::task::JoinHandle<()>>,
    from_reader_tx: tokio::sync::broadcast::Sender<(ConnectionState, RawPacket)>,
    to_writer: tokio::sync::mpsc::Sender<RawPacket>,
}

impl Connection {
    #[instrument(skip(tcp_stream))]
    fn new(tcp_stream: tokio::net::TcpStream, protocol_version: i32) -> Self {
        let address = tcp_stream.peer_addr().expect("Failed to get peer address");

        let (read_half, write_half) = tcp_stream.into_split();

        let state = Arc::new(SharedState::new(ConnectionState::Handshaking));

        let (reader_tx, _) = tokio::sync::broadcast::channel(1024);

        let read_state = state.clone();
        let read_packet_tx = reader_tx.clone();

        let span = tracing::info_span!("read_task", stats = "");
        let read_span = span.clone();
        let read_future = async move {
            let mut read_stream = CountingStream::new(read_half);
            loop {
                read_span.record("stats", &format!("{:?}", read_stream.stats));

                match Self::read_packet_raw_static(&mut read_stream).await {
                    Ok(packet) => {
                        let state = read_state.get();

                        match get_packet(packet.packet_id, state, PacketDirection::Clientbound) {
                            Some(descriptor) => {
                                log::debug!("Received packet: {:?}", descriptor.name);
                            }
                            None => {
                                log::debug!("Received unknown packet: {:?}", packet.packet_id);
                            }
                        }

                        read_packet_tx
                            .send((state, packet))
                            .expect("Failed to send packet");
                    }
                    Err(e) => {
                        warn!("Failed to read packet length: {:?}", e);
                        warn!("Closing read thread");
                        break;
                    }
                }
            }
        }
        .instrument(span);
        let read_task = Arc::new(tokio::spawn(read_future));

        let (writer_tx, mut reader_rx) = tokio::sync::mpsc::channel::<RawPacket>(1024);

        let write_state = state.clone();
        let write_future = async move {
            let mut write_stream = CountingStream::new(write_half);
            loop {
                match reader_rx.recv().await {
                    Some(packet) => {
                        let state = write_state.get();
                        match get_packet(packet.packet_id, state, PacketDirection::Serverbound) {
                            Some(descriptor) => {
                                log::debug!("Sending packet: {:?}", descriptor.name);
                            }
                            None => {
                                log::debug!("Sending unknown packet: {:?}", packet.packet_id);
                            }
                        }

                        let mut buf = Vec::new();
                        packet.write(&mut buf).expect("Failed to write packet");
                        let len = VarInt(buf.len() as i32);
                        <VarInt as AsyncPacketWritable>::write(len, &mut write_stream)
                            .await
                            .expect("Failed to write packet length");
                        write_stream
                            .write_all(&buf)
                            .await
                            .expect("Failed to write packet data");
                    }
                    None => {
                        warn!("Failed to receive packet from channel");
                        warn!("Closing write thread");
                        break;
                    }
                }
            }
        }
        .instrument(tracing::info_span!("write_task"));
        let write_task = Arc::new(tokio::spawn(write_future));

        Self {
            protocol_version,
            address,
            state,
            read_task,
            write_task,
            from_reader_tx: reader_tx,
            to_writer: writer_tx,
        }
    }

    #[instrument]
    pub async fn status(mut self) -> std::io::Result<StatusResponseJson> {
        if self.state.get() != ConnectionState::Handshaking {
            error!("Invalid state for status request: {:?}", self.state);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid state for status request",
            ));
        }

        let server_address = self.address.ip().to_string();
        let server_port = self.address.port();

        let handshake = protocol::serverbound::HandshakePacket::new(
            self.protocol_version,
            &server_address,
            server_port,
            protocol::serverbound::HandshakeNextState::Status,
        );
        let status_request = protocol::serverbound::StatusRequestPacket::new();
        self.send_packet(handshake).await?;
        self.state.set(ConnectionState::Status);
        self.send_packet(status_request).await?;

        let response = self
            .wait_for::<protocol::clientbound::StatusResponsePacket>()
            .await?;

        Ok(response.json)
    }

    pub async fn login(mut self) -> std::io::Result<()> {
        if self.state.get() != ConnectionState::Handshaking {
            error!("Invalid state for login request: {:?}", self.state);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid state for login request",
            ));
        }

        let server_address = self.address.ip().to_string();
        let server_port = self.address.port();

        let handshake = protocol::serverbound::HandshakePacket::new(
            self.protocol_version,
            &server_address,
            server_port,
            protocol::serverbound::HandshakeNextState::Login,
        );
        self.send_packet(handshake).await?;
        self.state.set(ConnectionState::Login);
        let login_start = protocol::serverbound::LoginStartPacket::new("volcano", 0u128.into());
        self.send_packet(login_start).await?;

        let success = self
            .wait_for::<protocol::clientbound::LoginSuccessPacket>()
            .await?;

        log::debug!("Login success: {:?}", success);

        let login_acknowledged = protocol::serverbound::LoginAcknowledgedPacket::new();
        self.send_packet(login_acknowledged).await?;
        self.state.set(ConnectionState::Configuration);

        self.wait_for::<protocol::clientbound::FinishConfigurationPacket>()
            .await?;

        Ok(())
    }

    #[instrument]
    pub async fn send_packet<P: Packet>(&mut self, packet: P) -> std::io::Result<()> {
        let mut buf = Vec::new();
        packet.write(&mut buf)?;
        let raw_packet = RawPacket {
            packet_id: P::PACKET_ID,
            bytes: buf,
        };
        self.to_writer.send(raw_packet).await.map_err(|e| {
            error!("Failed to send packet: {:?}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to send packet")
        })
    }

    #[instrument]
    pub fn blocking_wait_for<P: Packet>(&mut self) -> std::io::Result<P> {
        loop {
            let mut rx = self.from_reader_tx.subscribe();
            match rx.blocking_recv() {
                Ok((state, packet)) => {
                    if state == P::PACKET_STATE && packet.packet_id == P::PACKET_ID {
                        return P::read(&mut packet.bytes.as_slice());
                    }
                }
                Err(e) => {
                    error!("Failed to receive packet: {:?}", e);
                    break Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to receive packet",
                    ));
                }
            }
        }
    }

    #[instrument]
    pub async fn wait_for<P: Packet>(&mut self) -> std::io::Result<P> {
        let mut rx = self.from_reader_tx.subscribe();
        loop {
            match rx.recv().await {
                Ok((state, packet)) => {
                    if state == P::PACKET_STATE && packet.packet_id == P::PACKET_ID {
                        return P::read(&mut packet.bytes.as_slice());
                    }
                }
                Err(e) => {
                    error!("Failed to receive packet: {:?}", e);
                    break Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to receive packet",
                    ));
                }
            }
        }
    }

    #[instrument(skip(callback))]
    pub async fn on<P, F>(&self, mut callback: F) -> tokio::task::JoinHandle<()>
    where
        P: Packet + 'static,
        F: FnMut(P) + Send + 'static,
    {
        let mut rx = self.from_reader_tx.subscribe();
        let future = tokio::spawn(
            async move {
                loop {
                    match rx.recv().await {
                        Ok((state, packet)) => {
                            if state == P::PACKET_STATE && packet.packet_id == P::PACKET_ID {
                                match P::read(&mut packet.bytes.as_slice()) {
                                    Ok(typed) => callback(typed),
                                    Err(e) => error!("Failed to read packet: {:?}", e),
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive packet: {:?}", e);
                            break;
                        }
                    }
                }
            }
            .instrument(tracing::info_span!("packet_listener")),
        );

        future
    }

    #[instrument(skip(read))]
    fn blocking_read_packet_raw_static<R: Read>(read: &mut R) -> std::io::Result<RawPacket> {
        let len = <VarInt as PacketReadable>::read(read)?;
        let mut data = vec![0; len.0 as usize];
        read.read_exact(&mut data)?;
        let packet = RawPacket::read(&mut data.as_slice())?;
        Ok(packet)
    }

    #[instrument(skip(read))]
    async fn read_packet_raw_static<R>(read: &mut R) -> std::io::Result<RawPacket>
    where
        R: AsyncReadExt + Unpin,
    {
        let len = <VarInt as AsyncPacketReadable>::read(read).await?;
        let mut data = vec![0; len.0 as usize];
        read.read_exact(&mut data).await?;
        let packet = RawPacket::read(&mut data.as_slice())?;
        Ok(packet)
    }
}

impl TryClone for Connection {
    fn try_clone(&self) -> Result<Self, ()> {
        Ok(Self {
            protocol_version: self.protocol_version,
            address: self.address,
            state: self.state.clone(),
            read_task: self.read_task.clone(),
            write_task: self.write_task.clone(),
            from_reader_tx: self.from_reader_tx.clone(),
            to_writer: self.to_writer.clone(),
        })
    }
}

#[derive(Debug)]
struct SharedState {
    state: AtomicU8,
}

impl SharedState {
    fn new(state: ConnectionState) -> Self {
        Self {
            state: AtomicU8::new(state as u8),
        }
    }

    fn get(&self) -> ConnectionState {
        self.state.load(std::sync::atomic::Ordering::Relaxed).into()
    }

    fn set(&self, state: ConnectionState) {
        let prev = self.get();
        log::debug!("State {:?} -> {:?}", prev, state);
        self.state
            .store(state as u8, std::sync::atomic::Ordering::Relaxed);
    }
}
