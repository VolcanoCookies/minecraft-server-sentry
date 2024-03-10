use std::{fmt::Debug, io::Cursor, ops::Deref, pin::Pin};

use futures::{future, Future, FutureExt, TryFutureExt};
use log::trace;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    net::TcpStream,
};
use tracing::instrument;

use crate::packet_types::{PacketType, VarInt};

pub trait Packet {
    const PACKET_ID: i32;
}

pub trait ClientPacket: Packet
where
    Self: Sized,
    Self: 'static,
    Self: Packet,
    Self: Debug,
{
    fn deserialize(raw_packet: RawPacket) -> std::io::Result<Self>;

    fn receive(
        stream: &mut TcpStream,
    ) -> Pin<Box<dyn Future<Output = std::io::Result<Self>> + '_>> {
        let raw_packet = RawPacket::from_reader(stream);
        Box::pin(raw_packet.and_then(|result| future::ready(Self::deserialize(result))))
    }
}

pub trait ServerPacket: Packet
where
    Self: Sized,
    Self: Debug,
{
    fn serialize(self) -> std::io::Result<RawPacket>;
}

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub packet_id: i32,
    pub bytes: Vec<u8>,
}

impl RawPacket {
    pub fn new(packet_id: i32) -> Self {
        Self {
            packet_id,
            bytes: Vec::new(),
        }
    }

    pub fn write<P: PacketType>(&mut self, field: P) {
        field.write(&mut self.bytes);
    }

    pub fn write_vec<P: PacketType>(&mut self, fields: Vec<P>) {
        P::write_array(fields, &mut self.bytes);
    }

    pub fn cursor(self) -> Cursor<Vec<u8>> {
        Cursor::new(self.bytes)
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let packet_id = VarInt(self.packet_id);

        let data = [packet_id.to_bytes(), self.bytes].concat();
        let len = VarInt(data.len() as i32);

        [len.to_bytes(), data].concat()
    }

    #[instrument]
    pub async fn from_reader<R: AsyncRead + Unpin + Debug>(
        reader: &mut R,
    ) -> std::io::Result<RawPacket> {
        trace!("Reading packet from stream");
        let len = VarInt::read_async(reader).await?;
        trace!("Packet length: {}", len);

        let mut data = vec![0; len as usize];
        reader.read_exact(&mut data).await?;

        let mut cursor = Cursor::new(data);

        let packet_id = VarInt::read(&mut cursor)?;
        trace!("Packet ID: {}", packet_id);
        let mut bytes = Vec::new();
        match std::io::Read::read_to_end(&mut cursor, &mut bytes) {
            Ok(_) => (),
            Err(e) => {
                trace!("Error reading packet: {:?}", e);
                return Err(e);
            }
        }
        trace!("Packet bytes: {:?}", bytes.clone());

        Ok(Self { packet_id, bytes })
    }
}
