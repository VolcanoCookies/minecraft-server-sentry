use core::panic;
use std::{io::Bytes, slice::Iter};

use tokio::{io::AsyncReadExt, net::TcpStream};

const SEGMENT_BITS: u32 = 0x7F;
const CONTINUE_BIT: u32 = 0x80;

#[derive(Debug)]
pub struct VarInt {
    pub bytes: Vec<u8>,
}

impl VarInt {
    pub fn new(int: i32) -> Self {
        let mut uint = int as u32;
        let mut bytes = Vec::<u8>::new();
        loop {
            if (uint & !SEGMENT_BITS) == 0 {
                bytes.push(uint as u8);
                break Self { bytes };
            }

            bytes.push(((uint & SEGMENT_BITS) | CONTINUE_BIT) as u8);

            // Note: >>> means that the sign bit is shifted with the rest of the number rather than being left alone
            uint >>= 7;
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub async fn parse_from_stream(stream: &mut TcpStream) -> std::io::Result<i32> {
        let mut value = 0;
        let mut position = 0;

        let i = loop {
            let currentByte = stream.read_u8().await?;
            value |= (currentByte as u32 & SEGMENT_BITS) << position;

            if (currentByte as u32 & CONTINUE_BIT) == 0 {
                break value;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        };

        Ok(i as i32)
    }

    pub fn parse(bytes_iter: &mut Iter<u8>) -> i32 {
        let mut value = 0;
        let mut position = 0;

        let i = loop {
            let current_byte = bytes_iter.next().unwrap().to_owned();
            value |= (current_byte as u32 & SEGMENT_BITS) << position;

            if (current_byte as u32 & CONTINUE_BIT) == 0 {
                break value;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        };

        i as i32
    }
}

pub fn parse_bool(iter: &mut Iter<u8>) -> bool {
    match iter.next().expect("No more bytes") {
        0x00 => false,
        0x01 => true,
        other => panic!("Invalid byte value: {}", other),
    }
}

pub fn parse_varint(iter: &mut Iter<u8>) -> i32 {
    VarInt::parse(iter)
}

pub fn parse_string(iter: &mut Iter<u8>) -> String {
    let len = parse_varint(iter);
    let raw_str: Vec<u8> = iter.by_ref().cloned().take((len * 4) as usize).collect();
    std::str::from_utf8(&raw_str).unwrap().to_owned()
}

pub fn parse_bytes(iter: &mut Iter<u8>, len: usize) -> Vec<u8> {
    let bytes: Vec<u8> = iter.by_ref().cloned().take(len).collect();
    bytes
}
