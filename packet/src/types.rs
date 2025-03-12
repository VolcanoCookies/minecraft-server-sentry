use core::panic;
use std::io::Read;

use crate::{read::PacketReadable, write::PacketWritable};

#[derive(Debug)]
#[repr(transparent)]
pub struct VarInt(pub i32);

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> i32 {
        value.0
    }
}

impl VarInt {
    pub const SEGMENT_BITS: u32 = 0x7F;
    pub const CONTINUE_BIT: u32 = 0x80;

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut uint = self.0 as u32;
        let mut bytes = Vec::<u8>::new();
        loop {
            if (uint & !Self::SEGMENT_BITS) == 0 {
                bytes.push(uint as u8);
                break bytes;
            }

            bytes.push((uint & Self::SEGMENT_BITS | Self::CONTINUE_BIT) as u8);

            // Note: >>> means that the sign bit is shifted with the rest of the number rather than being left alone
            uint >>= 7;
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> i32 {
        let mut value = 0;
        let mut position = 0;

        for byte in bytes.iter() {
            let current_byte = *byte;
            value |= (current_byte as u32 & Self::SEGMENT_BITS) << position;

            if (current_byte as u32 & Self::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        }

        value as i32
    }
}

impl PacketReadable for VarInt {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut buf = [0; 1];

        loop {
            reader.read_exact(&mut buf)?;
            let current_byte = buf[0] as u32;
            value |= (current_byte & Self::SEGMENT_BITS) << position;

            if (current_byte & Self::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        }

        Ok(VarInt(value as i32))
    }
}

impl PacketWritable for VarInt {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let bytes = self.to_bytes();
        writer.write_all(&bytes)?;
        Ok(())
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct UUID(pub u128);

impl From<u128> for UUID {
    fn from(value: u128) -> Self {
        UUID(value)
    }
}

impl From<UUID> for u128 {
    fn from(value: UUID) -> u128 {
        value.0
    }
}

impl PacketReadable for UUID {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf)?;
        let value = u128::from_be_bytes(buf);
        Ok(UUID(value))
    }
}

impl PacketWritable for UUID {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, BufWriter, Write};

    use crate::{read::PacketReadable, types::VarInt, write::PacketWritable};

    #[test]
    fn encode_decode_varint() {
        let test_values: [(i32, Vec<u8>); 11] = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2097151, vec![0xff, 0xff, 0x7f]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ];

        for (val, bytes) in test_values.iter() {
            println!("Testing value: {}", val);
            let varint = VarInt(*val);
            {
                let mut varint_bytes = Vec::new();
                {
                    let mut writer = BufWriter::new(&mut varint_bytes);
                    varint.write(&mut writer).unwrap();
                    writer.flush().unwrap();
                }

                assert_eq!(
                    *bytes, *varint_bytes,
                    "Expected: {:?}, got: {:?}",
                    bytes, varint_bytes
                );
            }

            {
                let bytes = bytes.to_vec();
                let mut reader = BufReader::new(bytes.as_slice());
                let decoded = VarInt::read(&mut reader).unwrap();

                assert_eq!(*val, decoded.0, "Expected: {}, got: {}", val, decoded.0);
            }
        }
    }
}
