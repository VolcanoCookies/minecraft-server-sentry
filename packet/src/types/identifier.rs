use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{read::PacketReadable, write::PacketWritable};

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Identifier {
    pub fn new(name: &str) -> Self {
        Identifier(name.to_string())
    }

    pub fn namespace(&self) -> &str {
        self.0.split_once(':').map_or("minecraft", |(ns, _)| ns)
    }

    pub fn value(&self) -> &str {
        self.0.split_once(':').map_or(&self.0, |(_, value)| value)
    }
}

impl PacketReadable for Identifier {
    fn read<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let raw: String = PacketReadable::read(reader)?;
        Ok(Identifier(raw))
    }
}

impl PacketWritable for Identifier {
    fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        PacketWritable::write(&self.0, writer)
    }
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Identifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
