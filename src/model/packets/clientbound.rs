use std::slice::Iter;

use tokio::net::TcpStream;

use crate::{
    model::uuid::UUID,
    packet::Packet,
    types::{parse_bool, parse_bytes, parse_string, parse_varint},
};

pub struct EncryptionRequestPacket {
    pub server_id: String,
    pub public_key_length: i32,
    pub public_key: Vec<u8>,
    pub verify_token_length: i32,
    pub verify_token: Vec<u8>,
}

impl EncryptionRequestPacket {
    const packet_id: i32 = 0x01;

    pub async fn parse(stream: &mut TcpStream) -> std::io::Result<Self> {
        let packet = Packet::from_stream(stream).await?;

        assert_eq!(Self::packet_id, packet.packet_id);
        let mut bytes_iter = packet.bytes.iter();

        let server_id = parse_string(&mut bytes_iter);
        let public_key_length = parse_varint(&mut bytes_iter);
        let public_key = parse_bytes(&mut bytes_iter, public_key_length as usize);
        let verify_token_length = parse_varint(&mut bytes_iter);
        let verify_token = parse_bytes(&mut bytes_iter, verify_token_length as usize);

        Ok(Self {
            server_id,
            public_key_length,
            public_key,
            verify_token_length,
            verify_token,
        })
    }
}

pub struct LoginSuccessPacket {
    pub uuid: String,
    pub username: String,
    pub number_of_properties: i32,
    pub properties: Vec<Property>,
}

impl LoginSuccessPacket {
    const packet_id: i32 = 0x02;

    pub async fn parse(stream: &mut TcpStream) -> std::io::Result<Self> {
        let packet = Packet::from_stream(stream).await?;

        assert_eq!(Self::packet_id, packet.packet_id);
        let mut bytes_iter = packet.bytes.iter();

        let uuid = parse_string(&mut bytes_iter);
        let username = parse_string(&mut bytes_iter);
        let number_of_properties = parse_varint(&mut bytes_iter);
        let mut properties = Vec::new();

        for i in 0..number_of_properties {
            properties.push(Property::parse(&mut bytes_iter)?);
        }

        Ok(Self {
            uuid,
            username,
            number_of_properties,
            properties,
        })
    }
}

pub struct Property {
    pub name: String,
    pub value: String,
    pub is_signed: bool,
    pub signature: Option<String>,
}

impl Property {
    pub fn parse(bytes_iter: &mut Iter<'_, u8>) -> std::io::Result<Self> {
        let name = parse_string(bytes_iter);
        let value = parse_string(bytes_iter);
        let is_signed = parse_bool(bytes_iter);

        let signature = if is_signed {
            Some(parse_string(bytes_iter))
        } else {
            None
        };

        Ok(Self {
            name,
            value,
            is_signed,
            signature,
        })
    }
}
