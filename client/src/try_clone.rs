pub trait TryClone: Sized {
    fn try_clone(&self) -> Result<Self, ()>;
}

impl TryClone for std::net::TcpStream {
    fn try_clone(&self) -> Result<Self, ()> {
        self.try_clone().map_err(|_| ())
    }
}
