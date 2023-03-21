use std::io::Write;

use crate::types::VarInt;

pub struct Packet {
    bytes: Vec<u8>,
}

impl Packet {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn writeVarInt(&mut self, int: i32) {
        self.bytes.append(&mut VarInt::new(int).bytes);
    }

    pub fn writeString(&mut self, str: String) {
        self.writeVarInt(str.len() as i32);
        self.bytes.append(&mut str.into_bytes());
    }

    pub fn writeShort(&mut self, short: i16) {
        self.bytes.write(&short.to_be_bytes());
    }

    pub fn writeByte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    pub fn to_bytes(mut self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.append(&mut VarInt::new(self.bytes.len() as i32).bytes);
        bytes.append(&mut self.bytes);
        bytes
    }
}

pub fn handshake_packet(ip: &str, port: i16) -> Packet {
    let mut packet = Packet::new();

    packet.writeVarInt(0);
    packet.writeVarInt(-1);
    packet.writeString(ip.to_owned());
    packet.writeShort(port);
    packet.writeVarInt(1);

    packet
}

pub fn status_request_packet() -> Packet {
    let mut packet = Packet::new();

    packet.writeVarInt(0);

    packet
}
