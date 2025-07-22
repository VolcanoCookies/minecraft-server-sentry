use std::io::Write;

use crate::types::VarInt;

pub trait PacketWritable {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()>;
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.write(&mut buffer).unwrap();
        buffer
    }
    fn index(&self) -> i32 {
        0
    }
}

impl<T: PacketWritable> PacketWritable for &T {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        (*self).write(writer)
    }
}

impl PacketWritable for u8 {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[*self])?;
        Ok(())
    }
}

impl PacketWritable for i8 {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[*self as u8])?;
        Ok(())
    }
}

macro_rules! impl_packet_writable {
    ($type:ty) => {
        impl PacketWritable for $type {
            fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                writer.write_all(&self.to_be_bytes())?;
                Ok(())
            }
        }
    };
}

impl_packet_writable!(u16);
impl_packet_writable!(i16);
impl_packet_writable!(u32);
impl_packet_writable!(i32);
impl_packet_writable!(u64);
impl_packet_writable!(i64);
impl_packet_writable!(f32);
impl_packet_writable!(f64);

impl PacketWritable for bool {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let byte: u8 = if *self { 0x01 } else { 0x00 };
        byte.write(writer)?;
        Ok(())
    }
}

impl PacketWritable for String {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = VarInt::from(self.len() as i32);
        len.write(writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl<T> PacketWritable for Vec<T>
where
    T: PacketWritable,
{
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = VarInt::from(self.len() as i32);
        len.write(writer)?;

        for item in self {
            item.write(writer)?;
        }

        Ok(())
    }
}

impl<T> PacketWritable for Option<T>
where
    T: PacketWritable,
{
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Some(value) => {
                true.write(writer)?;
                value.write(writer)
            }
            None => false.write(writer),
        }
    }
}

impl<T, const C: usize> PacketWritable for [T; C]
where
    T: PacketWritable,
{
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}
