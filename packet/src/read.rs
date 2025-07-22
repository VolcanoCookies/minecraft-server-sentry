use std::{io::Read, mem::MaybeUninit};

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

impl PacketReadable for i8 {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
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
impl_packet_readable!(f32);
impl_packet_readable!(f64);

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
        String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
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

// Assume always present
// Optional comes from cond attribute not packet readable trait
impl<T> PacketReadable for Option<T>
where
    T: PacketReadable,
{
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(Some(T::read(reader)?))
    }
}

impl<T: PacketReadable, const C: usize> PacketReadable for [T; C] {
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut array: [MaybeUninit<T>; C] = unsafe { MaybeUninit::uninit().assume_init() };

        for elem in &mut array[..] {
            let value = T::read(reader)?;
            *elem = MaybeUninit::new(value);
        }

        let array = unsafe { std::mem::transmute_copy::<_, [T; C]>(&array) };
        Ok(array)
    }
}
