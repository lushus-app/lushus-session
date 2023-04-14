#[derive(Debug, thiserror::Error)]
pub enum RedisError {
    #[error("Redis connection error: {0}")]
    ConnectionError(String),
    #[error("Redis query error: {0}")]
    QueryError(String),
}
