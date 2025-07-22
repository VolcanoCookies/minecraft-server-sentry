use std::collections::HashMap;

use crate::{read::PacketReadable, write::PacketWritable};

#[derive(Debug, PartialEq)]
pub struct NBT {
    pub name: String,
    pub value: NBTValue,
}

impl NBT {
    pub fn new(name: String, value: NBTValue) -> Self {
        Self { name, value }
    }

    fn read<R: std::io::Read>(reader: &mut R, skip_name: bool) -> std::io::Result<Self> {
        let mut tag_buf = [0u8; 1];
        reader.read_exact(&mut tag_buf)?;
        let tag = tag_buf[0];

        if tag > 12 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid NBT tag: {}", tag),
            ));
        }

        let name = if skip_name {
            "".to_string()
        } else {
            read_mutf_8(reader)?
        };

        let value = NBTValue::read_untagged(tag, reader)?;

        Ok(NBT::new(name, value))
    }

    fn write<W: std::io::Write>(&self, writer: &mut W, skip_name: bool) -> std::io::Result<()> {
        let tag = self.value.to_type_id();
        writer.write_all(&[tag])?;
        if !skip_name {
            write_mutf_8(writer, &self.name)?;
        }
        self.value.write_untagged(writer)?;

        Ok(())
    }
}

impl PacketReadable for NBT {
    fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Self::read(reader, true)
    }
}

impl PacketWritable for NBT {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.write(writer, true)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum NBTValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NBTValue>),
    Compound(HashMap<String, NBTValue>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

fn write_mutf_8<W: std::io::Write>(writer: &mut W, string: &str) -> std::io::Result<()> {
    if string.is_empty() {
        writer.write_all(&[0, 0, 0])?;
        return Ok(());
    }

    let mut buf = Vec::new();

    for char in string.chars() {
        if char == '\u{0000}' {
            buf.push(0b1100_0000);
            buf.push(0b1000_0000);
        } else if char <= '\u{007F}' {
            buf.push(0b1000_0000 | (char as u8));
        } else if char <= '\u{07FF}' {
            buf.push(0b1100_0000 | ((char as u16) >> 6) as u8);
            buf.push(0b1000_0000 | ((char as u8) & 0b0011_1111));
        } else {
            buf.push(0b1110_0000 | ((char as u16) >> 12) as u8);
            buf.push(0b1000_0000 | (((char as u16) >> 6) as u8 & 0b0011_1111));
            buf.push(0b1000_0000 | ((char as u8) & 0b0011_1111));
        }
    }

    let len = buf.len() as u16;

    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(&buf)?;
    Ok(())
}

fn read_mutf_8<R: std::io::Read>(reader: &mut R) -> std::io::Result<String> {
    let mut len_buf = [0u8; 2];
    reader.read_exact(&mut len_buf)?;
    let len = u16::from_be_bytes(len_buf);
    if len == 0 {
        reader.read_exact(&mut [0u8; 1])?;
        return Ok(String::new());
    }

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;

    let mut string = String::new();
    let mut i = 0;
    while i < buf.len() {
        let byte = buf[i];
        if byte & 0b1000_0000 == 0 {
            string.push(char::from(byte));
            i += 1;
        } else if byte & 0b1110_0000 == 0b1100_0000 {
            let c1 = (byte & 0b0001_1111) as u16;
            let c2 = (buf[i + 1] & 0b0011_1111) as u16;
            if let Some(c) = char::from_u32(((c1 << 6) | c2) as u32) {
                string.push(c);
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid MUTF-8 string",
                ));
            }
            i += 2;
        } else if byte & 0b1111_0000 == 0b1110_0000 {
            let c1 = (byte & 0b0000_1111) as u16;
            let c2 = (buf[i + 1] & 0b0011_1111) as u16;
            let c3 = (buf[i + 2] & 0b0011_1111) as u16;
            if let Some(c) = char::from_u32(((c1 << 12) | (c2 << 6) | c3) as u32) {
                string.push(c);
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid MUTF-8 string",
                ));
            }
            i += 3;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid MUTF-8 string",
            ));
        }
    }

    Ok(string)
}

impl NBTValue {
    pub fn to_type_id(&self) -> u8 {
        match self {
            NBTValue::Byte(_) => 1,
            NBTValue::Short(_) => 2,
            NBTValue::Int(_) => 3,
            NBTValue::Long(_) => 4,
            NBTValue::Float(_) => 5,
            NBTValue::Double(_) => 6,
            NBTValue::ByteArray(_) => 7,
            NBTValue::String(_) => 8,
            NBTValue::List(_) => 9,
            NBTValue::Compound(_) => 10,
            NBTValue::IntArray(_) => 11,
            NBTValue::LongArray(_) => 12,
        }
    }

    fn write_tagged<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let tag = self.to_type_id();
        writer.write_all(&[tag])?;
        self.write_untagged(writer)?;
        Ok(())
    }

    fn write_untagged<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            NBTValue::Byte(i8) => {
                writer.write_all(&[*i8 as u8])?;
            }
            NBTValue::Short(i16) => {
                writer.write_all(&i16.to_be_bytes())?;
            }
            NBTValue::Int(i32) => {
                writer.write_all(&i32.to_be_bytes())?;
            }
            NBTValue::Long(i64) => {
                writer.write_all(&i64.to_be_bytes())?;
            }
            NBTValue::Float(f32) => {
                writer.write_all(&f32.to_be_bytes())?;
            }
            NBTValue::Double(f64) => {
                writer.write_all(&f64.to_be_bytes())?;
            }
            NBTValue::ByteArray(vec) => {
                let len = vec.len() as i32;
                writer.write_all(&len.to_be_bytes())?;
                for i in vec {
                    writer.write_all(&[*i as u8])?;
                }
            }
            NBTValue::String(string) => {
                write_mutf_8(writer, string)?;
            }
            NBTValue::List(items) => {
                let len = items.len() as i32;
                writer.write_all(&len.to_be_bytes())?;
                if items.is_empty() {
                    writer.write_all(&[0])?;
                    return Ok(());
                }
                let type_id = items[0].to_type_id();
                writer.write_all(&[type_id])?;
                for nbt in items {
                    if nbt.to_type_id() != type_id {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "List elements must be of the same type",
                        ));
                    }
                    nbt.write_untagged(writer)?;
                }
            }
            NBTValue::Compound(items) => {
                for (name, value) in items {
                    let type_id = value.to_type_id();
                    writer.write_all(&[type_id])?;
                    write_mutf_8(writer, name)?;
                    value.write_untagged(writer)?;
                }
                writer.write_all(&[0])?;
            }
            NBTValue::IntArray(ints) => {
                let len = ints.len() as i32;
                writer.write_all(&len.to_be_bytes())?;
                for i in ints {
                    writer.write_all(&i.to_be_bytes())?;
                }
            }
            NBTValue::LongArray(longs) => {
                let len = longs.len() as i32;
                writer.write_all(&len.to_be_bytes())?;
                for i in longs {
                    writer.write_all(&i.to_be_bytes())?;
                }
            }
        }

        Ok(())
    }

    fn read_tagged<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut tag_buf = [0u8; 1];
        reader.read_exact(&mut tag_buf)?;
        let tag = tag_buf[0];
        if tag < 0 || tag > 12 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid NBT tag: {}", tag),
            ));
        }
        Self::read_untagged(tag, reader)
    }

    fn read_untagged<R: std::io::Read>(tag: u8, reader: &mut R) -> std::io::Result<Self> {
        match tag {
            0 => todo!(),
            1 => {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                Ok(NBTValue::Byte(buf[0] as i8))
            }
            2 => {
                let mut buf = [0u8; 2];
                reader.read_exact(&mut buf)?;
                Ok(NBTValue::Short(i16::from_be_bytes(buf)))
            }
            3 => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                Ok(NBTValue::Int(i32::from_be_bytes(buf)))
            }
            4 => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                Ok(NBTValue::Long(i64::from_be_bytes(buf)))
            }
            5 => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                Ok(NBTValue::Float(f32::from_be_bytes(buf)))
            }
            6 => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                Ok(NBTValue::Double(f64::from_be_bytes(buf)))
            }
            7 => {
                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)?;
                let len = i32::from_be_bytes(len_buf);
                let mut vec = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let mut buf = [0u8; 1];
                    reader.read_exact(&mut buf)?;
                    vec.push(buf[0] as i8);
                }
                Ok(NBTValue::ByteArray(vec))
            }
            8 => {
                let string = read_mutf_8(reader)?;
                Ok(NBTValue::String(string))
            }
            9 => {
                let mut type_buf = [0u8; 1];
                reader.read_exact(&mut type_buf)?;

                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)?;
                let len = i32::from_be_bytes(len_buf);
                let mut items = Vec::with_capacity(len as usize);
                if len == 0 {
                    return Ok(NBTValue::List(items));
                }

                for _ in 0..len {
                    items.push(Self::read_untagged(type_buf[0], reader)?);
                }
                Ok(NBTValue::List(items))
            }
            10 => {
                let mut items = HashMap::new();
                loop {
                    let mut type_buf = [0u8; 1];
                    reader.read_exact(&mut type_buf)?;
                    if type_buf[0] == 0 {
                        break;
                    }
                    let name = read_mutf_8(reader)?;
                    let value = Self::read_untagged(type_buf[0], reader)?;
                    items.insert(name, value);
                }
                Ok(NBTValue::Compound(items))
            }
            11 => {
                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)?;
                let len = i32::from_be_bytes(len_buf);
                let mut vec = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let mut buf = [0u8; 4];
                    reader.read_exact(&mut buf)?;
                    vec.push(i32::from_be_bytes(buf));
                }
                Ok(NBTValue::IntArray(vec))
            }
            12 => {
                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)?;
                let len = i32::from_be_bytes(len_buf);
                let mut vec = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let mut buf = [0u8; 8];
                    reader.read_exact(&mut buf)?;
                    vec.push(i64::from_be_bytes(buf));
                }
                Ok(NBTValue::LongArray(vec))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid NBT tag: {}", tag),
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::types::{
        NBTValue::{self},
        NBT,
    };

    #[test]
    fn nbt_hello_world() {
        let bytes = include_bytes!("../../samples/hello_world.nbt");
        let mut cursor = std::io::Cursor::new(bytes);
        let nbt = crate::types::NBT::read(&mut cursor, false).unwrap();

        let NBT { name, value: nbt } = nbt;
        assert_eq!(name, "hello world");

        match nbt {
            NBTValue::Compound(items) => {
                assert_eq!(name, "hello world");
                assert_eq!(items.len(), 1);
                let value = items.get("name").unwrap();
                match value {
                    NBTValue::String(string) => {
                        assert_eq!(string, "Bananrama");
                    }
                    _ => panic!("Expected string"),
                }
            }
            _ => panic!("Expected compound"),
        }
    }

    #[test]
    fn nbt_big_test() {
        let bytes = include_bytes!("../../samples/bigtest.nbt");
        let mut cursor = std::io::Cursor::new(bytes);
        let nbt = crate::types::NBT::read(&mut cursor, false).unwrap();

        /* let expected = NBT {
            name: "Level".to_string(),
            value: NBTValue::Compound(

            )
        };

        panic!("Big test"); */
    }
}
