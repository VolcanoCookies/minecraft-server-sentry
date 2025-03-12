use std::fmt::Debug;

use read::PacketReadable;
use write::PacketWritable;

pub mod raw;
pub mod read;
pub mod registry;
pub mod types;
pub mod write;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ConnectionState {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    Configuration = 4,
    Play = 5,
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::Handshaking
    }
}

impl From<u8> for ConnectionState {
    fn from(value: u8) -> Self {
        match value {
            0 => ConnectionState::Handshaking,
            1 => ConnectionState::Status,
            2 => ConnectionState::Login,
            3 => ConnectionState::Transfer,
            4 => ConnectionState::Configuration,
            5 => ConnectionState::Play,
            _ => ConnectionState::Handshaking,
        }
    }
}

impl From<ConnectionState> for u8 {
    fn from(value: ConnectionState) -> Self {
        match value {
            ConnectionState::Handshaking => 0,
            ConnectionState::Status => 1,
            ConnectionState::Login => 2,
            ConnectionState::Transfer => 3,
            ConnectionState::Configuration => 4,
            ConnectionState::Play => 5,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PacketDirection {
    Clientbound,
    Serverbound,
    Both,
}

pub trait Packet: PacketReadable + PacketWritable + Debug {
    const PACKET_ID: i32;
    const PACKET_STATE: ConnectionState;
    const PACKET_DIRECTION: PacketDirection;

    /* fn read_packet_boxed<R: std::io::Read>(reader: &mut R) -> std::io::Result<Box<dyn Packet>>
    where
        Self: Sized,
    {
        Ok(Box::new(Self::read_packet(reader)?))
    } */
    fn read_packet<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized;
    fn write_packet<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()>;
}
