use std::{fmt::Display, io::Read};

use crate::{read::PacketReadable, write::PacketWritable};

pub trait Vectorable: Display + PacketReadable + PacketWritable {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedBitset<const C: usize> {
    data: Vec<u8>,
}

impl<const C: usize> Default for FixedBitset<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const C: usize> FixedBitset<C> {
    pub fn new() -> Self {
        FixedBitset {
            data: vec![0; (C + 7) / 8],
        }
    }

    pub fn set(&mut self, index: usize) {
        if index < C {
            let byte_index = index / 8;
            let bit_index = index % 8;
            self.data[byte_index] |= 1 << bit_index;
        }
    }

    pub fn get(&self, index: usize) -> bool {
        if index < C {
            let byte_index = index / 8;
            let bit_index = index % 8;
            (self.data[byte_index] & (1 << bit_index)) != 0
        } else {
            false
        }
    }
}

impl<const C: usize> PacketReadable for FixedBitset<C> {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = vec![0; (C + 7) / 8];
        reader.read_exact(&mut data)?;
        Ok(FixedBitset { data })
    }
}

impl<const C: usize> PacketWritable for FixedBitset<C> {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.data)?;
        Ok(())
    }
}
