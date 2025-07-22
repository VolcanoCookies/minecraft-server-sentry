use std::io::Read;

use crate::{read::PacketReadable, write::PacketWritable};

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
