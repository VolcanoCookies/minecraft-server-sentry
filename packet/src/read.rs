use std::io::Read;

use crate::types::VarInt;

pub trait PacketReadable {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized;
}

impl PacketReadable for u8 {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

macro_rules! impl_packet_readable {
    ($type:ty) => {
        impl PacketReadable for $type {
            fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
                let mut buf = [0; std::mem::size_of::<$type>()];
                reader.read_exact(&mut buf)?;
                Ok(<$type>::from_be_bytes(buf))
            }
        }
    };
}

impl_packet_readable!(u16);
impl_packet_readable!(i16);
impl_packet_readable!(u32);
impl_packet_readable!(i32);
impl_packet_readable!(u64);
impl_packet_readable!(i64);

impl PacketReadable for bool {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let byte = u8::read(reader)?;
        match byte {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid byte value",
            )),
        }
    }
}

impl PacketReadable for String {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let len: VarInt = PacketReadable::read(reader)?;
        let mut buf = vec![0; len.0 as usize];
        reader.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
}

impl<T> PacketReadable for Vec<T>
where
    T: PacketReadable,
{
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let len = VarInt::read(reader)?;
        let mut vec = Vec::with_capacity(len.0 as usize);

        for _ in 0..len.0 {
            vec.push(T::read(reader)?);
        }

        Ok(vec)
    }
}

impl<T> PacketReadable for Option<T>
where
    T: PacketReadable,
{
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let present = bool::read(reader)?;
        if present {
            Ok(Some(T::read(reader)?))
        } else {
            Ok(None)
        }
    }
}
