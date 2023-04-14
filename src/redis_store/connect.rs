use redis::aio::ConnectionManager;

pub trait Connect {
    fn connection(&self) -> &ConnectionManager;
}
