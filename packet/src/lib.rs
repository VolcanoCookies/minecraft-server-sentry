use std::{
    any::Any,
    fmt::{Debug, Display},
};

use read::PacketReadable;
use write::PacketWritable;

pub mod async_read;
pub mod async_write;
pub mod raw;
pub mod read;
pub mod registry;
pub mod types;
pub mod write;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum ConnectionState {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
    Configuration = 4,
    Play = 5,
}

impl Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Handshaking => write!(f, "Handshaking"),
            ConnectionState::Status => write!(f, "Status"),
            ConnectionState::Login => write!(f, "Login"),
            ConnectionState::Transfer => write!(f, "Transfer"),
            ConnectionState::Configuration => write!(f, "Configuration"),
            ConnectionState::Play => write!(f, "Play"),
        }
    }
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

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub enum PacketDirection {
    Clientbound,
    Serverbound,
    Both,
}

pub trait Packet: PacketReadable + PacketWritable + Debug {
    const PACKET_ID: i32;
    const PACKET_STATE: ConnectionState;
    const PACKET_DIRECTION: PacketDirection;
    const PACKET_DESCRIPTOR: registry::PacketDescriptor;

    fn read_packet<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized;
    fn write_packet<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

pub trait PacketData: Any + Debug {
    fn packet_id(&self) -> i32;
    fn packet_state(&self) -> ConnectionState;
    fn packet_direction(&self) -> PacketDirection;
    fn descriptor(&self) -> &'static registry::PacketDescriptor;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}
