use tokio::io::AsyncReadExt;

pub trait PacketReadableAsync: Sized + Send {
    async fn read_async<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: AsyncReadExt + Unpin;
}
