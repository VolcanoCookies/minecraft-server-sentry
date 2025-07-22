use tokio::io::AsyncWriteExt;

pub trait PacketWritableAsync: Sized + Send {
    async fn write_async<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: AsyncWriteExt + Unpin;
}
