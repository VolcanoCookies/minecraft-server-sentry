use std::{
    fmt::Debug,
    io::{Read, Result, Write},
    ops::Deref,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::{instrument, trace};

const SEGMENT_BITS: u32 = 0x7F;
const CONTINUE_BIT: u32 = 0x80;

#[derive(Debug)]
pub struct VarInt(pub i32);

#[derive(Debug)]
pub struct VarLong(pub i64);

#[derive(Debug)]
pub struct ByteArray(pub Vec<u8>);

impl VarInt {
    pub async fn read_async<R: AsyncRead + Unpin>(reader: &mut R) -> Result<i32> {
        let mut value = 0;
        let mut position = 0;

        let i = loop {
            let current_byte = reader.read_u8().await?;
            value |= (current_byte as u32 & SEGMENT_BITS) << position;

            if (current_byte as u32 & CONTINUE_BIT) == 0 {
                break value;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        };

        Ok(i as i32)
    }
}

impl Deref for ByteArray {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn read<R: Read + Debug, O: PacketType + PacketType<Output = B>, B: PacketType>(
    reader: &mut R,
) -> Result<B> {
    O::read(reader)
}

pub trait PacketType: Sized {
    type Output;

    fn read<R: Read + Debug>(reader: &mut R) -> Result<Self::Output>;

    fn read_array<R: Read + Debug>(reader: &mut R, size: usize) -> Result<Vec<Self::Output>> {
        let mut out = Vec::with_capacity(size);
        for _ in 0..size {
            out.push(Self::read(reader)?);
        }
        Ok(out)
    }

    fn write<W: Write + Debug>(self, writer: &mut W);

    fn write_array<W: Write + Debug>(arr: Vec<Self>, writer: &mut W) {
        for e in arr {
            Self::write(e, writer);
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.write(&mut bytes);
        bytes.flush();
        bytes
    }
}

impl PacketType for bool {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        match reader.read_u8()? {
            0x00 => Ok(false),
            0x01 => Ok(true),
            other => panic!("Unexpected byte {} parsing bool", other),
        }
    }

    fn write<W: Write>(self, writer: &mut W) {
        if self {
            writer.write(&[0x01]);
        } else {
            writer.write(&[0x00]);
        }
    }
}

impl PacketType for i8 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_i8()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_i8(self);
    }
}

impl PacketType for u8 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_u8()
    }

    fn read_array<R: Read>(reader: &mut R, size: usize) -> Result<Vec<Self>> {
        let mut buf = vec![0u8; size];
        reader.read_exact(&mut buf);
        Ok(buf)
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_u8(self);
    }

    fn write_array<W: Write>(arr: Vec<Self>, writer: &mut W) {
        writer.write_all(&arr);
    }
}

impl PacketType for i16 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_i16::<BigEndian>()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_i16::<BigEndian>(self);
    }
}

impl PacketType for u16 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_u16::<BigEndian>()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_u16::<BigEndian>(self);
    }
}

impl PacketType for i32 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_i32::<BigEndian>()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_i32::<BigEndian>(self);
    }
}
impl PacketType for i64 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_i64::<BigEndian>()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_i64::<BigEndian>(self);
    }
}
impl PacketType for u128 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_u128::<BigEndian>()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_u128::<BigEndian>(self);
    }
}

impl PacketType for f32 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_f32::<BigEndian>()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_f32::<BigEndian>(self);
    }
}

impl PacketType for f64 {
    type Output = Self;

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        reader.read_f64::<BigEndian>()
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_f64::<BigEndian>(self);
    }
}

impl PacketType for String {
    type Output = Self;

    #[instrument(target = "parsing", level = "trace")]
    fn read<R: Read + Debug>(reader: &mut R) -> Result<Self> {
        trace!("Reading string");
        let len = VarInt::read(reader)?;
        trace!("String length: {}", len);
        // Multiply by 4 since len denotes characters and each character is 4 bytes
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(buf.as_mut_slice())?;
        trace!("String bytes: {:?}", buf);
        Ok(std::str::from_utf8(&buf).unwrap().to_owned())
    }

    fn write<W: Write + Debug>(self, writer: &mut W) {
        let len = self.len() as i32;
        VarInt(len).write(writer);
        writer.write_all(self.as_bytes());
    }
}

pub fn read_chat<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

pub fn read_identifier<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

impl PacketType for VarInt {
    type Output = i32;

    fn read<R: Read>(reader: &mut R) -> Result<Self::Output> {
        let mut value = 0;
        let mut position = 0;

        let i = loop {
            let current_byte = reader.read_u8()?;
            value |= (current_byte as u32 & SEGMENT_BITS) << position;

            if (current_byte as u32 & CONTINUE_BIT) == 0 {
                break value;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        };

        Ok(i as i32)
    }

    fn write<W: Write>(self, writer: &mut W) {
        let mut left = self.0 as i32;
        loop {
            if (left & !SEGMENT_BITS as i32) == 0 {
                writer.write_u8(left as u8);
                break;
            }
            writer.write_u8(((left & SEGMENT_BITS as i32) | CONTINUE_BIT as i32) as u8);
            left >>= 7;
        }
    }
}

impl PacketType for VarLong {
    type Output = i64;

    fn read<R: Read>(reader: &mut R) -> Result<Self::Output> {
        let mut value = 0;
        let mut position = 0;

        let l = loop {
            let current_byte = reader.read_u8()?;
            value |= (current_byte as u64 & SEGMENT_BITS as u64) << position;

            if (current_byte as u32 & CONTINUE_BIT) == 0 {
                break value;
            }

            position += 7;

            if position >= 32 {
                panic!("VarLong is too big");
            }
        };

        Ok(l as i64)
    }

    fn write<W: Write>(self, writer: &mut W) {
        writer.write_i64::<BigEndian>(self.0);
    }
}

impl PacketType for ByteArray {
    type Output = ByteArray;

    fn read<R: Read + Debug>(reader: &mut R) -> Result<Self::Output> {
        let len = VarInt::read(reader)?;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(buf.as_mut_slice())?;
        Ok(Self(buf))
    }

    fn write<W: Write + Debug>(self, writer: &mut W) {
        let len = self.0.len() as i32;
        VarInt(len).write(writer);
        writer.write_all(self.0.as_slice());
    }
}

pub fn read_entity_metadata<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

pub fn read_slot<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

pub fn read_NBT_tag<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

pub fn read_position<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

pub fn read_angle<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

pub fn read_uuid<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

pub fn read_enum<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    todo!("Implement");
}

impl Into<VarInt> for i32 {
    fn into(self) -> VarInt {
        VarInt(self)
    }
}

impl Into<VarLong> for i64 {
    fn into(self) -> VarLong {
        VarLong(self)
    }
}
