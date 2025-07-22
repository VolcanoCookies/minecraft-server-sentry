use std::{fmt::Display, io::Read};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    async_read::PacketReadableAsync, async_write::PacketWritableAsync, read::PacketReadable,
    write::PacketWritable,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl From<i64> for Position {
    fn from(value: i64) -> Self {
        let x = (value >> 38) as i32;
        let y = (value << 52 >> 52) as i32;
        let z = (value << 26 >> 38) as i32;

        Position { x, y, z }
    }
}

impl From<Position> for i64 {
    fn from(value: Position) -> i64 {
        ((value.x as i64 & 0x3FFFFFF) << 38)
            | ((value.z as i64 & 0x3FFFFFF) << 12)
            | (value.y as i64 & 0xFFF)
    }
}

impl PacketReadable for Position {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buffer = [0; 8];
        reader.read_exact(&mut buffer)?;
        let raw = i64::from_be_bytes(buffer);
        Ok(Position::from(raw))
    }
}

impl PacketWritable for Position {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let raw: i64 = (*self).into();
        writer.write(&raw.to_be_bytes())?;
        Ok(())
    }
}

impl PacketReadableAsync for Position {
    async fn read_async<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: AsyncReadExt + Unpin,
    {
        let raw: i64 = reader.read_i64().await?;
        Ok(Position::from(raw))
    }
}

impl PacketWritableAsync for Position {
    async fn write_async<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        let raw: i64 = (*self).into();
        writer.write_all(&raw.to_be_bytes()).await?;
        Ok(())
    }
}
