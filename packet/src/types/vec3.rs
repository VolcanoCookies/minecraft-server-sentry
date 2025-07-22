use std::{fmt::Display, io::Read};

use crate::{read::PacketReadable, write::PacketWritable};

pub trait Vectorable: Display + PacketReadable + PacketWritable {}

impl<T> Vectorable for T where T: Display + PacketReadable + PacketWritable {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3<I>
where
    I: Vectorable,
{
    pub x: I,
    pub y: I,
    pub z: I,
}

impl<I: Vectorable> Display for Vec3<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl<I: Vectorable> Vec3<I> {
    pub fn new(x: I, y: I, z: I) -> Self {
        Vec3 { x, y, z }
    }
}

impl<I: Vectorable> PacketReadable for Vec3<I> {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let x = I::read(reader)?;
        let y = I::read(reader)?;
        let z = I::read(reader)?;
        Ok(Vec3 { x, y, z })
    }
}

impl<I: Vectorable> PacketWritable for Vec3<I> {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.x.write(writer)?;
        self.y.write(writer)?;
        self.z.write(writer)?;
        Ok(())
    }
}
