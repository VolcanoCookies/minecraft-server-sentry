use packet::types::{VarInt, UUID};
use packet_macros::{Packet, PacketReadable, PacketWritable};

#[derive(PacketReadable, PacketWritable, Debug)]
#[packet(repr = "VarInt")]
pub enum HandshakeNextState {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Handshaking)]
pub struct HandshakePacket {
    #[packet(repr = "VarInt")]
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: HandshakeNextState,
}

impl HandshakePacket {
    pub fn new(
        protocol_version: i32,
        server_address: &str,
        server_port: u16,
        next_state: HandshakeNextState,
    ) -> Self {
        Self {
            protocol_version: protocol_version,
            server_address: server_address.to_owned(),
            server_port,
            next_state,
        }
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Status)]
pub struct StatusRequestPacket {}

impl StatusRequestPacket {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x01)]
#[packet_state(Status)]
pub struct PingRequestPacket {
    pub timestamp: i64,
}

impl PingRequestPacket {
    pub fn new(timestamp: i64) -> Self {
        Self { timestamp }
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x01)]
#[packet_state(Login)]
pub struct EncryptionResponsePacket {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl EncryptionResponsePacket {
    pub fn new(shared_secret: Vec<u8>, verify_token: Vec<u8>) -> Self {
        Self {
            shared_secret,
            verify_token,
        }
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x03)]
#[packet_state(Login)]
pub struct LoginAcknowledgedPacket {}

impl LoginAcknowledgedPacket {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Packet, Debug)]
#[packet_id(0x01)]
#[packet_state(Login)]
pub struct LoginStartPacket {
    pub name: String,
    pub uuid: UUID,
}

impl LoginStartPacket {
    pub fn new(name: &str, uuid: UUID) -> Self {
        Self {
            name: name.to_owned(),
            uuid,
        }
    }
}

#[cfg(test)]
mod tests {
    use packet::Packet;

    use crate::serverbound::StatusRequestPacket;

    #[test]
    fn write_status_request_packet() {
        let packet = StatusRequestPacket::new();
        let mut bytes = vec![];
        packet.write_packet(&mut bytes).unwrap();
        assert_eq!(bytes, vec![0x01, 0x00]);
    }
}
