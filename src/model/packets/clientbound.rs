use std::{fmt::Debug, io::Read};

use packet_macros::client_packet;
use serde_json::Result;

use crate::{
    packet::Packet,
    packet_types::{ByteArray, PacketType},
    response::ResponseData,
};

#[client_packet(id = 0x00)]
pub struct StatusResponsePacket {
    pub data: String,
}

impl StatusResponsePacket {
    pub fn get_json(&self) -> Result<ResponseData> {
        serde_json::from_str(&self.data)
    }
}

#[client_packet(id = 0x01)]
pub struct EncryptionRequestPacket {
    pub server_id: String,
    pub public_key: ByteArray,
    pub verify_token: ByteArray,
}

pub struct LoginSuccessPacket {
    pub uuid: String,
    pub username: String,
    pub number_of_properties: i32,
    pub properties: Vec<Property>,
}

pub struct Property {
    pub name: String,
    pub value: String,
    pub is_signed: bool,
    pub signature: Option<String>,
}
