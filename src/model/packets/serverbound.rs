use packet_macros::server_packet;

use crate::packet_types::VarInt;

#[server_packet(id = 0x00)]
pub struct HandshakePacket {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: VarInt,
}

#[server_packet(id = 0x00)]
pub struct StatusRequestPacket {}

#[server_packet(id = 0x00)]
pub struct LoginRequestPacket {
    pub username: String,
    pub uuid: u128,
}
