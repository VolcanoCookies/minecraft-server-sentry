use std::slice::Iter;

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

    pub fn parse(bytes_iter: &mut Iter<u8>) -> i32 {
        let mut value = 0;
        let mut position = 0;

        let i = loop {
            let currentByte = bytes_iter.next().unwrap().to_owned();
            value |= (currentByte as u32 & SEGMENT_BITS) << position;

            if (currentByte as u32 & CONTINUE_BIT) == 0 {
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
