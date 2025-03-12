use std::io::{Read, Write};

use tracing::{debug, instrument};

use crate::{read::PacketReadable, types::VarInt, write::PacketWritable};

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub packet_id: i32,
    pub bytes: Vec<u8>,
}

impl PacketReadable for RawPacket {
    #[instrument(level = "trace", skip(reader))]
    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let packet_id = VarInt::read(reader)?.0;
        debug!("Packet ID: {:?}", packet_id);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        debug!("Packet bytes: {:?}", bytes.len());

        Ok(Self { packet_id, bytes })
    }
}

impl PacketWritable for RawPacket {
    #[instrument(level = "trace", skip(writer))]
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let packet_id = VarInt(self.packet_id);
        packet_id.write(writer)?;
        writer.write_all(&self.bytes)?;

        Ok(())
    }
}

impl RawPacket {
    pub fn new(packet_id: i32) -> Self {
        Self {
            packet_id,
            bytes: Vec::new(),
        }
    }
}
