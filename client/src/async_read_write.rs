use packet::types::VarInt;

pub trait AsyncPacketReadable {
    async fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: tokio::io::AsyncRead + Unpin,
        Self: Sized;
}

impl AsyncPacketReadable for VarInt {
    async fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: tokio::io::AsyncReadExt + Unpin,
    {
        let mut value = 0;
        let mut position = 0;

        loop {
            let current_byte = reader.read_u8().await? as u32;
            value |= (current_byte & Self::SEGMENT_BITS) << position;

            if (current_byte & Self::CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        }

        Ok(VarInt(value as i32))
    }
}

pub trait AsyncPacketWritable {
    async fn write<W>(self, writer: &mut W) -> std::io::Result<()>
    where
        W: tokio::io::AsyncWrite + Unpin,
        Self: Sized;
}

impl AsyncPacketWritable for VarInt {
    async fn write<W>(self, writer: &mut W) -> std::io::Result<()>
    where
        W: tokio::io::AsyncWriteExt + Unpin,
    {
        let mut value = self.0 as u32;

        loop {
            if (value & !VarInt::SEGMENT_BITS) == 0 {
                writer.write_u8(value as u8).await?;
                break;
            }

            writer
                .write_u8((value & VarInt::SEGMENT_BITS | VarInt::CONTINUE_BIT) as u8)
                .await?;

            value >>= 7;
        }

        Ok(())
    }
}
