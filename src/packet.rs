use std::io::Write;

use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::types::VarInt;

pub struct Packet {
    pub packet_id: i32,
    pub bytes: Vec<u8>,
}

impl Packet {
    pub fn new(packet_id: i32) -> Self {
        Self {
            packet_id,
            bytes: Vec::new(),
        }
    }

    pub fn writeVarInt(&mut self, int: i32) {
        self.bytes.append(&mut VarInt::new(int).bytes);
    }

    pub fn writeString(&mut self, str: &str) {
        self.writeVarInt(str.len() as i32);
        self.bytes.append(&mut str.as_bytes().to_vec());
    }

    pub fn writeShort(&mut self, short: i16) {
        self.bytes.write(&short.to_be_bytes());
    }

    pub fn writeByte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    pub fn writeBool(&mut self, b: bool) {
        if b {
            self.writeByte(0x01);
        } else {
            self.writeByte(0x00);
        }
    }

    pub fn to_bytes(mut self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        bytes.append(&mut VarInt::new(self.packet_id).bytes);
        bytes.append(&mut self.bytes);

        [VarInt::new(bytes.len() as i32).bytes, bytes].concat()
    }

    pub async fn from_stream(stream: &mut TcpStream) -> std::io::Result<Packet> {
        let len = VarInt::parse_from_stream(stream).await?;
        let mut data = Vec::with_capacity(len as usize);

        stream.read_exact(&mut data).await?;
        let mut data_iter = data.iter();

        let packet_id = VarInt::parse(&mut data_iter);
        let bytes = data_iter.cloned().collect();

        Ok(Self { packet_id, bytes })
    }
}

pub fn handshake_status_packet(ip: &str, port: i16) -> Packet {
    handskake_packet(ip, port, 1)
}

pub fn handshake_login_packet(ip: &str, port: i16) -> Packet {
    handskake_packet(ip, port, 2)
}

pub fn handskake_packet(ip: &str, port: i16, next_state: i32) -> Packet {
    let mut packet = Packet::new(0);

    packet.writeVarInt(-1);
    packet.writeString(ip);
    packet.writeShort(port);
    packet.writeVarInt(next_state);

    packet
}

pub fn status_request_packet() -> Packet {
    let mut packet = Packet::new(0);

    packet
}

pub fn login_start(username: &str, uuid: &str) -> Packet {
    let mut packet = Packet::new(0);

    packet.writeString(username);
    packet.writeBool(true);
    packet.writeString(uuid);

    packet
}
