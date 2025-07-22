use std::io::{Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::instrument;

use crate::{
    async_read::PacketReadableAsync, async_write::PacketWritableAsync, read::PacketReadable,
    registry::PacketDescriptor, types::VarInt, write::PacketWritable,
};

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub packet_id: i32,
    pub bytes: Vec<u8>,
    pub descriptor: Option<&'static PacketDescriptor>,
}

impl RawPacket {
    pub fn new(packet_id: i32) -> Self {
        Self {
            packet_id,
            bytes: Vec::new(),
            descriptor: None,
        }
    }

    #[instrument(level = "trace", skip(reader))]
    pub fn read_blocking<R>(reader: &mut R, threshold: i32) -> std::io::Result<Self>
    where
        R: Read,
    {
        // Read length of packet
        let len = VarInt::read(reader)?.0;

        // Read packet data
        let mut packet_buf = vec![0; len as usize];
        reader.read_exact(&mut packet_buf)?;

        // Compression
        let buf = if threshold >= 0 {
            let data_len = VarInt::read(reader)?.0;

            if data_len > 0 {
                let mut uncompressed_data = vec![0; data_len as usize];
                ZlibDecoder::new(packet_buf.as_slice()).read_exact(&mut uncompressed_data)?;
                uncompressed_data
            } else {
                packet_buf
            }
        } else {
            packet_buf
        };

        let mut data = buf.as_slice();
        // Read packet id
        let packet_id = VarInt::read(&mut data)?.0;
        // Rest of the packet is the data
        let bytes = data.to_vec();

        Ok(Self {
            packet_id,
            bytes,
            descriptor: None,
        })
    }

    #[instrument(level = "trace", skip(writer))]
    pub fn write_blocking<W>(&self, writer: &mut W, threshold: i32) -> std::io::Result<()>
    where
        W: Write,
    {
        // Write packet it to separate buffer
        let mut packet_id_buf = Vec::with_capacity(5);
        let packet_id = VarInt(self.packet_id);
        packet_id.write(&mut packet_id_buf)?;

        // Get length of packet id + packet data
        let len = packet_id_buf.len() + self.bytes.len();

        if threshold >= 0 {
            if len >= threshold as usize {
                let mut compressed_data = Vec::new();
                {
                    let mut compressor = ZlibEncoder::new(
                        compressed_data.as_mut_slice(),
                        flate2::Compression::default(),
                    );
                    compressor.write_all(&packet_id_buf)?;
                    compressor.write_all(&self.bytes)?;
                }

                // Write packet length
                let compressed_len = VarInt(compressed_data.len() as i32);
                let mut compressed_len_buf = Vec::with_capacity(5);
                compressed_len.write(&mut compressed_len_buf)?;

                let len = compressed_len_buf.len() + compressed_data.len();
                let len_varint = VarInt(len as i32);
                len_varint.write(writer)?;
                writer.write_all(&compressed_len_buf)?;
                writer.write_all(&compressed_data)?;
            } else {
                let len_varint = VarInt(len as i32);
                len_varint.write(writer)?;
                writer.write(&mut [0u8; 1])?;
                writer.write_all(&packet_id_buf)?;
                writer.write_all(&self.bytes)?;
            }
        } else {
            let len_varint = VarInt(len as i32);

            len_varint.write(writer)?;
            writer.write_all(&packet_id_buf)?;
            writer.write_all(&self.bytes)?;
        }

        Ok(())
    }

    /// Read a packet from a reader asynchronously.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read the packet from.
    /// * `threshold` - The compression threshold, no compression if less than 0.
    ///
    /// # Errors
    ///
    /// This function will return an error if the packet could not be read.
    #[instrument(level = "trace", skip(reader))]
    pub async fn read<R>(reader: &mut R, threshold: i32) -> std::io::Result<Self>
    where
        R: AsyncReadExt + Unpin,
    {
        // Read length of packet
        let len = VarInt::read_async(reader).await?.0;

        log::trace!("Reading packet with length {:?}", len);

        let mut packet_buf = vec![0; len as usize];
        reader.read_exact(&mut packet_buf).await?;

        log::trace!("Read packet data");

        // Compression
        let buf = if threshold >= 0 {
            let mut data = packet_buf.as_slice();
            let data_len = VarInt::read(&mut data)?.0;

            if data_len > 0 {
                log::trace!("Decompressed length {:?}", data_len);
                let mut uncompressed_data = vec![0; data_len as usize];
                ZlibDecoder::new(data).read_exact(&mut uncompressed_data)?;
                uncompressed_data
            } else {
                log::trace!("Reading uncompressed data");
                data.to_vec()
            }
        } else {
            log::trace!("No compression");
            packet_buf
        };

        let mut data = buf.as_slice();
        // Read packet id
        let packet_id = VarInt::read_async(&mut data).await?.0;
        log::trace!("Read packet id {:?}", packet_id);
        // Rest of the packet is the data
        let bytes = data.to_vec();
        log::trace!("Read packet data");

        Ok(Self {
            packet_id,
            bytes,
            descriptor: None,
        })
    }

    #[instrument(level = "trace", skip(writer))]
    pub async fn write<W>(&self, writer: &mut W, threshold: i32) -> std::io::Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        // Write packet it to separate buffer
        let mut packet_id_buf = Vec::with_capacity(5);
        log::trace!("Writing packet id {:?}", self.packet_id);
        let packet_id = VarInt(self.packet_id);
        packet_id.write(&mut packet_id_buf)?;

        // Get length of packet id + packet data
        let len = packet_id_buf.len() + self.bytes.len();
        log::trace!("Packet length {:?}", len);

        if threshold >= 0 {
            if len >= threshold as usize {
                log::trace!("Compressing packet");
                let mut compressed_data = Vec::new();
                {
                    let mut compressor = ZlibEncoder::new(
                        compressed_data.as_mut_slice(),
                        flate2::Compression::default(),
                    );
                    compressor.write_all(&packet_id_buf)?;
                    compressor.write_all(&self.bytes)?;
                }

                // Write packet length
                let compressed_len = VarInt(compressed_data.len() as i32);
                let mut compressed_len_buf = Vec::with_capacity(5);
                compressed_len.write(&mut compressed_len_buf)?;

                let len = compressed_len_buf.len() + compressed_data.len();
                let len_varint = VarInt(len as i32);
                len_varint.write_async(writer).await?;
                writer.write_all(&compressed_len_buf).await?;
                writer.write_all(&compressed_data).await?;
            } else {
                log::trace!("Writing uncompressed packet");
                let len_varint = VarInt(len as i32 + 1);
                len_varint.write_async(writer).await?;
                writer.write_u8(0).await?;
                writer.write_all(&packet_id_buf).await?;
                writer.write_all(&self.bytes).await?;
            }
        } else {
            log::trace!("Compression disabled");
            let len_varint = VarInt(len as i32);

            len_varint.write_async(writer).await?;
            writer.write_all(&packet_id_buf).await?;
            writer.write_all(&self.bytes).await?;
        }

        Ok(())
    }
}
