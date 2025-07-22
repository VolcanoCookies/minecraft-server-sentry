use core::panic;
use std::{
    io::Read,
    ops::{Deref, DerefMut},
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    async_read::PacketReadableAsync, async_write::PacketWritableAsync, read::PacketReadable,
    write::PacketWritable,
};

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

impl PacketReadableAsync for VarInt {
    async fn read_async<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: AsyncReadExt + Unpin,
    {
        let mut value = 0;
        let mut position = 0;

        loop {
            let current_byte = reader.read_u8().await? as u32;
            value |= (current_byte & VarInt::SEGMENT_BITS) << position;

            if (current_byte & VarInt::CONTINUE_BIT) == 0 {
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

impl PacketWritableAsync for VarInt {
    async fn write_async<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        let bytes = self.to_bytes();
        writer.write_all(&bytes).await?;
        Ok(())
    }
}

impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VarInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
