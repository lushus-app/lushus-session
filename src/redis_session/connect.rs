use redis::aio::ConnectionManager;

use crate::session_store::configuration::Configuration;

pub trait Connect {
    fn config(&self) -> &Configuration;
    fn connection(&self) -> &ConnectionManager;
}
