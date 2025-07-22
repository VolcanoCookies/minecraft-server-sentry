use std::io::{Read, Write};

use crate::{read::PacketReadable, write::PacketWritable};

#[derive(Debug)]
#[repr(transparent)]
pub struct GreedyVec<T>(pub Vec<T>);

impl<T> From<Vec<T>> for GreedyVec<T> {
    fn from(value: Vec<T>) -> Self {
        GreedyVec(value)
    }
}

impl<T> From<GreedyVec<T>> for Vec<T> {
    fn from(value: GreedyVec<T>) -> Vec<T> {
        value.0
    }
}

impl<T: PacketReadable> PacketReadable for GreedyVec<T> {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut vec = Vec::new();

        loop {
            match T::read(reader) {
                Ok(value) => vec.push(value),
                Err(_) => break,
            }
        }

        Ok(GreedyVec(vec))
    }
}

impl<T: PacketWritable> PacketWritable for GreedyVec<T> {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for value in self.0.iter() {
            value.write(writer)?;
        }

        Ok(())
    }
}
