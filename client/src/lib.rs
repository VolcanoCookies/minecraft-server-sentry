#[macro_use]
pub mod macros;
pub mod stream;
mod try_clone;

use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicI32, AtomicU8},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

use packet::{
    raw::RawPacket, read::PacketReadable, registry::get_packet, types::fixed_bitset::FixedBitset,
    write::PacketWritable, ConnectionState, Packet, PacketData, PacketDirection,
};
use protocol::{
    clientbound::{DisconnectPacket, LoginSuccessPacket, SetCompressionPacket, StatusResponseJson},
    serverbound::{
        configuration::{ChatMode, MainHand, ParticleStatus},
        play::ChatMessagePacket,
    },
};
use rand::random;
use stream::CountingStream;
use tokio::{
    io::AsyncWriteExt,
    net::tcp::OwnedReadHalf,
    sync::{Barrier, Notify},
};
use tracing::{error, instrument, warn, Instrument};
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

#[derive(Debug, Clone)]
pub struct Connection {
    protocol_version: i32,
    address: SocketAddr,
    state: Arc<SharedState>,
    read_task: Arc<tokio::task::JoinHandle<()>>,
    write_task: Arc<tokio::task::JoinHandle<()>>,
    from_reader_tx: tokio::sync::broadcast::Sender<(ConnectionState, RawPacket)>,
    to_writer: tokio::sync::mpsc::Sender<RawPacket>,
    compression: Arc<AtomicI32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum Compression {
    Unknown,
    Disabled,
    Enabled(i32),
}

impl From<i32> for Compression {
    fn from(value: i32) -> Self {
        if value == -1 {
            return Self::Unknown;
        } else if value == -2 {
            return Self::Disabled;
        } else if value >= 0 {
            return Self::Enabled(value);
        }
        panic!("Invalid compression value: {}", value);
    }
}

impl Into<i32> for Compression {
    fn into(self) -> i32 {
        match self {
            Self::Unknown => -1,
            Self::Disabled => -2,
            Self::Enabled(value) => value,
        }
    }
}

async fn read_packets(
    read: OwnedReadHalf,
    _span: tracing::Span,
    state: Arc<SharedState>,
    _compression: Arc<AtomicI32>,
    read_packet_tx: tokio::sync::broadcast::Sender<(ConnectionState, RawPacket)>,
) {
    let mut read = CountingStream::new(read);
    let mut compression: Compression = _compression
        .load(std::sync::atomic::Ordering::Relaxed)
        .into();
    loop {
        // Check compression until set
        if compression == Compression::Unknown {
            let new = _compression
                .load(std::sync::atomic::Ordering::Relaxed)
                .into();
            if new != Compression::Unknown {
                compression = new;
            }
        }

        match RawPacket::read(&mut read, compression.into()).await {
            Ok(mut packet) => {
                let state = state.get();

                match get_packet(packet.packet_id, state, PacketDirection::Clientbound) {
                    Some(descriptor) => {
                        log::debug!("[{}] Received packet: {:?}", state, descriptor.name);
                        packet.descriptor = Some(descriptor);
                    }
                    None => {
                        log::debug!(
                            "[{}] Received unknown packet: {:#04x}",
                            state,
                            packet.packet_id
                        );
                    }
                }

                if packet.descriptor == Some(&SetCompressionPacket::PACKET_DESCRIPTOR) {
                    let packet = SetCompressionPacket::read(&mut packet.bytes.as_slice())
                        .expect("Failed to read SetCompressionPacket");
                    _compression.store(packet.threshold, std::sync::atomic::Ordering::Relaxed);
                    compression = packet.threshold.into();
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

async fn write_packets(
    write: tokio::net::tcp::OwnedWriteHalf,
    state: Arc<SharedState>,
    _compression: Arc<AtomicI32>,
    mut reader_rx: tokio::sync::mpsc::Receiver<RawPacket>,
) {
    let write = CountingStream::new(write);
    let mut compression: Compression = _compression
        .load(std::sync::atomic::Ordering::Relaxed)
        .into();

    let mut buf_write = tokio::io::BufWriter::new(write);

    loop {
        match reader_rx.recv().await {
            Some(packet) => {
                if compression == Compression::Unknown {
                    let new = _compression
                        .load(std::sync::atomic::Ordering::Relaxed)
                        .into();
                    if new != Compression::Unknown {
                        compression = new;
                    }
                }

                let state = state.get();
                if let Some(descriptor) =
                    get_packet(packet.packet_id, state, PacketDirection::Serverbound)
                {
                    log::debug!("[{}] Sending packet: {:?}", state, descriptor.name);
                } else {
                    log::debug!(
                        "[{}] Sending unknown packet: {:#04x}",
                        state,
                        packet.packet_id
                    );
                }

                match packet.write(&mut buf_write, compression.into()).await {
                    Ok(_) => {
                        buf_write
                            .flush()
                            .await
                            .expect("Failed to flush write buffer");
                    }
                    Err(e) => {
                        warn!("Failed to write packet: {:?}", e);
                        warn!("Closing write thread");
                        break;
                    }
                }
            }
            None => {
                warn!("Failed to receive packet from channel");
                warn!("Closing write thread");
                break;
            }
        }
    }
}

impl Connection {
    #[instrument(skip(tcp_stream))]
    fn new(tcp_stream: tokio::net::TcpStream, protocol_version: i32) -> Self {
        let address = tcp_stream.peer_addr().expect("Failed to get peer address");

        let (read_half, write_half) = tcp_stream.into_split();

        let state = Arc::new(SharedState::new(ConnectionState::Handshaking));

        let (reader_tx, _) = tokio::sync::broadcast::channel(1024);

        let compression = Arc::new(AtomicI32::new(-1));

        let span = tracing::info_span!("read_task", stats = "");
        let read_future = read_packets(
            read_half,
            span.clone(),
            state.clone(),
            compression.clone(),
            reader_tx.clone(),
        )
        .instrument(tracing::info_span!("read_task"));
        let read_task = Arc::new(tokio::spawn(read_future));

        let (writer_tx, reader_rx) = tokio::sync::mpsc::channel::<RawPacket>(1024);

        let write_future = write_packets(write_half, state.clone(), compression.clone(), reader_rx)
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
            compression,
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
        let login_start = protocol::serverbound::LoginStartPacket::new(
            "VolcanoHD",
            123279235262967524908973144513429206752u128.into(),
        );
        self.send_packet(login_start).await?;

        self.on::<protocol::clientbound::DisconnectPacket, _>(|mut _conn, packet| {
            log::error!("Disconnected: {:?}", packet.reason);
        });

        let success = self
            .wait_for::<protocol::clientbound::LoginSuccessPacket>()
            .await?;

        log::debug!("Login success: {:?}", success);

        let login_acknowledged = protocol::serverbound::LoginAcknowledgedPacket::new();
        self.send_packet(login_acknowledged).await?;
        self.state.set(ConnectionState::Configuration);

        let _configuration_keep_alive = self
            .on::<protocol::clientbound::configuration::KeepAlivePacket, _>(
                move |mut conn, packet| {
                    let response =
                        protocol::serverbound::configuration::KeepAlivePacket { id: packet.id };

                    tokio::spawn(async move {
                        conn.send_packet(response)
                            .await
                            .expect("Failed to send keep alive response");
                    });
                },
            );

        self.once::<protocol::clientbound::configuration::ServerPluginMessagePacket, _>(
            |mut conn, packet| {
                if packet.channel == "minecraft:brand" {
                    let response =
                        protocol::serverbound::configuration::ClientPluginMessagePacket {
                            channel: "minecraft:brand".to_string(),
                            data: "vanilla".to_string().to_bytes(),
                        };
                    tokio::spawn(async move {
                        conn.send_packet(response)
                            .await
                            .expect("Failed to send brand response");
                    });
                }
            },
        );

        self.once::<protocol::clientbound::configuration::KnownPacksPacket, _>(
            |mut conn, packet| {
                let known_packets = protocol::serverbound::configuration::KnownPacksPacket {
                    packs: packet.packs,
                };

                tokio::spawn(async move {
                    conn.send_packet(known_packets)
                        .await
                        .expect("Failed to send known packs response");
                });
            },
        );

        let notify = Arc::new(Notify::new());

        let notify_inner = notify.clone();
        self.once::<protocol::clientbound::configuration::FinishConfigurationPacket, _>(
            move |mut conn, _| {
                let finish_configuration =
                    protocol::serverbound::configuration::FinishConfigurationPacket {};

                let notify_inner = notify_inner.clone();
                tokio::spawn(async move {
                    conn.send_packet(finish_configuration)
                        .await
                        .expect("Failed to send finish configuration response");
                    conn.state.set(ConnectionState::Play);
                    notify_inner.notify_one();
                });
            },
        );

        let client_information = protocol::serverbound::configuration::ClientInformationPacket {
            locale: "en_US".to_string(),
            view_distance: 10,
            chat_mode: ChatMode::Enabled,
            chat_colors: true,
            displayed_skin_parts: 0x7f,
            main_hand: MainHand::Left,
            enable_text_filtering: false,
            allow_server_listings: true,
            particle_status: ParticleStatus::All,
        };
        self.send_packet(client_information).await?;

        let notify_inner = notify.clone();
        let mut self_inner = self.clone();
        tokio::spawn(async move {
            notify_inner.notified().await;
            let now = SystemTime::now();
            let duration = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Failed to get timestamp");
            let millis = duration.as_millis() as i64;
            let salt: i64 = random();
            let chat_message = ChatMessagePacket {
                message: "Hello, world!".to_string(),
                timestamp: millis,
                salt,
                signature: None,
                message_count: 0.into(),
                acknowledged: FixedBitset::default(),
                checksum: 0,
            };
            let _ = self_inner.send_packet(chat_message).await;
        });

        while let Ok(raw) = self.wait_for_raw().await {
            if raw.descriptor.is_none() {
                log::error!(
                    "[{}] Received raw packet with no descriptor: {:#04x}",
                    self.state.get(),
                    raw.packet_id
                );
            }
            let descriptor = raw.descriptor.expect("Missing packet descriptor");
            match (descriptor.read_fn)(&mut raw.bytes.as_slice()) {
                Ok(packet) => {
                    log::debug!("[{}] Received packet: {:?}", self.state.get(), packet);
                }
                Err(e) => {
                    error!("Failed to read packet: {:?}", e);
                    break;
                }
            }
        }

        self.wait_for::<protocol::clientbound::configuration::FinishConfigurationPacket>()
            .await?;

        Ok(())
    }

    #[instrument]
    pub async fn send_packet<P>(&mut self, packet: P) -> std::io::Result<()>
    where
        P: Packet + PacketData,
    {
        let mut buf = Vec::new();
        packet.write(&mut buf)?;
        let raw_packet = RawPacket {
            packet_id: P::PACKET_ID,
            bytes: buf,
            descriptor: Some(packet.descriptor()),
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
                Ok((_, packet)) => {
                    if packet.descriptor == Some(&P::PACKET_DESCRIPTOR) {
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
    pub async fn wait_for_raw(&mut self) -> std::io::Result<RawPacket> {
        let mut rx = self.from_reader_tx.subscribe();
        loop {
            match rx.recv().await {
                Ok((_, packet)) => {
                    return Ok(packet);
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
    pub async fn wait_for_raw_timeout(
        &mut self,
        timeout: Duration,
    ) -> std::io::Result<Option<RawPacket>> {
        let mut rx = self.from_reader_tx.subscribe();
        loop {
            match tokio::time::timeout(timeout, rx.recv()).await {
                Ok(Ok((_, packet))) => {
                    return Ok(Some(packet));
                }
                Ok(Err(e)) => {
                    error!("Failed to receive packet: {:?}", e);
                    break Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to receive packet",
                    ));
                }
                Err(_) => {
                    return Ok(None);
                }
            }
        }
    }

    #[instrument(skip(self, callback))]
    pub fn on<P, F>(&self, mut callback: F) -> tokio::task::JoinHandle<()>
    where
        P: Packet + 'static,
        F: FnMut(Connection, P) + Send + 'static,
    {
        let this = self.clone();
        let future = tokio::spawn(
            async move {
                let mut rx = this.from_reader_tx.subscribe();
                loop {
                    match rx.recv().await {
                        Ok((state, packet)) => {
                            if state == P::PACKET_STATE && packet.packet_id == P::PACKET_ID {
                                match P::read(&mut packet.bytes.as_slice()) {
                                    Ok(typed) => {
                                        callback(this.clone(), typed);
                                    }
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

    #[instrument(skip(self, callback))]
    pub fn once<P, F>(&self, mut callback: F) -> tokio::task::JoinHandle<()>
    where
        P: Packet + 'static,
        F: FnMut(Connection, P) + Send + 'static,
    {
        let this = self.clone();
        let future = tokio::spawn(
            async move {
                let mut rx = this.from_reader_tx.subscribe();
                loop {
                    match rx.recv().await {
                        Ok((state, packet)) => {
                            if state == P::PACKET_STATE && packet.packet_id == P::PACKET_ID {
                                match P::read(&mut packet.bytes.as_slice()) {
                                    Ok(typed) => {
                                        callback(this.clone(), typed);
                                        break;
                                    }
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
