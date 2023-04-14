mod commands;
mod connect;
mod error;
mod execute_command;
mod redis_database;

use std::time::Duration;

use commands::Command;
use connect::Connect;
use error::RedisError;
use execute_command::ExecuteCommand;
pub use redis_database::RedisDatabase;

use crate::{session::Session, session_state::SessionState, SessionKey, Store};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Storage error: Backend error {0}")]
    BackendError(String),
    #[error("Storage error: Unable to serialize or deserialize session from session key: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error(transparent)]
    RedisError(#[from] RedisError),
}

#[async_trait::async_trait(?Send)]
impl<T> Store for &T
where
    T: Connect + ExecuteCommand,
{
    type Error = StoreError;

    async fn load(&self, session_key: &SessionKey) -> Result<Option<Session>, Self::Error> {
        let cache_key = (self.config().key_gen())(session_key);
        let value = self
            .execute_command::<Option<String>>(Command::get(cache_key))
            .await
            .map_err(StoreError::from)?;
        let state = value
            .map(|v| serde_json::from_str::<SessionState>(&v))
            .transpose()
            .map_err(StoreError::SerializationError)?;
        let session = state.map(|state| Session::new(session_key.clone(), state));
        Ok(session)
    }

    async fn save(&self, session: &Session, timeout: Duration) -> Result<(), Self::Error> {
        let session_id = session.id();
        let cache_key = (self.config().key_gen())(session_id);
        let state: SessionState = session.into();
        let body = serde_json::to_string(&state).map_err(StoreError::SerializationError)?;
        self.execute_command(Command::set(cache_key, body, timeout))
            .await
            .map_err(StoreError::from)?;
        Ok(())
    }

    async fn update(&self, session: &Session, timeout: Duration) -> Result<(), Self::Error> {
        let session_id = session.id();
        let cache_key = (self.config().key_gen())(session_id);
        let state: SessionState = session.into();
        let body = serde_json::to_string(&state).map_err(StoreError::SerializationError)?;
        let value = self
            .execute_command(Command::update(cache_key, body, timeout))
            .await
            .map(redis::Value::from)
            .map_err(StoreError::from)?;
        match value {
            redis::Value::Okay => Ok(()),
            redis::Value::Nil => Err(StoreError::BackendError(
                "Update returned nil response data".to_string(),
            )),
            _ => Err(StoreError::BackendError(
                "Update returned invalid response data".to_string(),
            )),
        }
    }

    async fn destroy(&self, session_key: &SessionKey) -> Result<(), Self::Error> {
        let cache_key = (self.config().key_gen())(session_key);
        self.execute_command(Command::delete(cache_key))
            .await
            .map_err(StoreError::from)
    }

    async fn exists(&self, session_key: &SessionKey) -> Result<bool, Self::Error> {
        let cache_key = (self.config().key_gen())(session_key);
        let exists = self
            .execute_command::<u64>(Command::exists(cache_key))
            .await
            .map_err(StoreError::from)?;
        Ok(exists > 0)
    }

    async fn ttl(&self, session_key: &SessionKey) -> Result<Duration, Self::Error> {
        let cache_key = (self.config().key_gen())(session_key);
        let ttl = self
            .execute_command::<u64>(Command::ttl(cache_key))
            .await
            .map_err(StoreError::from)?;
        Ok(Duration::from_secs(ttl))
    }
}
