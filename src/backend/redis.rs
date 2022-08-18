use redis::aio::ConnectionManager;
use std::time::Duration;

use crate::backend::{Backend, SessionKey};
use crate::session_state::SessionState;

struct Configuration {
    key_gen: Box<dyn Fn(&dyn AsRef<str>) -> String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            key_gen: Box::new(|v| v.as_ref().to_owned()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RedisError {
    #[error("Redis connection error: {0}")]
    ConnectionError(String),
    #[error("Redis query error: {0}")]
    QueryError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Storage error: Backend error {0}")]
    BackendError(String),
    #[error("Storage error: Unable to serialize or deserialize session from session key: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error(transparent)]
    RedisError(#[from] RedisError),
}

struct RedisSessionStore {
    config: Configuration,
    connection: ConnectionManager,
}

impl RedisSessionStore {
    async fn new(url: &str) -> Result<Self, RedisError> {
        let config = Default::default();
        let client = redis::Client::open(url)
            .map_err(|e| e.to_string())
            .map_err(RedisError::ConnectionError)?;
        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| e.to_string())
            .map_err(RedisError::ConnectionError)?;
        Ok(Self { config, connection })
    }

    async fn execute_command<T: redis::FromRedisValue>(
        &self,
        cmd: &mut redis::Cmd,
    ) -> Result<T, RedisError> {
        let result = cmd
            .query_async(&mut self.connection.clone())
            .await
            .map_err(|e| e.to_string())
            .map_err(RedisError::QueryError)?;
        Ok(result)
    }
}

#[async_trait::async_trait(?Send)]
impl Backend for RedisSessionStore {
    type Error = StorageError;

    async fn load(&self, session_key: SessionKey) -> Result<Option<SessionState>, Self::Error> {
        let key_gen = &self.config.key_gen;
        let cache_key = key_gen(&session_key);
        let mut cmd = redis::cmd("GET").arg(&[&cache_key]).clone();

        let value = self
            .execute_command::<Option<String>>(&mut cmd)
            .await
            .map_err(StorageError::from)?;
        value
            .map(|v| serde_json::from_str(&v))
            .transpose()
            .map_err(StorageError::SerializationError)
    }

    async fn save(
        &self,
        session_state: SessionState,
        timeout: Duration,
    ) -> Result<SessionKey, Self::Error> {
        let session_key = SessionKey::generate();
        let key_gen = &self.config.key_gen;
        let cache_key = key_gen(&session_key);
        let body =
            serde_json::to_string(&session_state).map_err(StorageError::SerializationError)?;
        let mut cmd = redis::cmd("SET")
            .arg(&[
                &cache_key,
                &body,
                "NX",
                "EX",
                &format!("{}", timeout.as_secs()),
            ])
            .clone();

        self.execute_command(&mut cmd)
            .await
            .map_err(StorageError::from)?;

        Ok(session_key)
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        timeout: Duration,
    ) -> Result<(), Self::Error> {
        let key_gen = &self.config.key_gen;
        let cache_key = key_gen(&session_key);
        let body =
            serde_json::to_string(&session_state).map_err(StorageError::SerializationError)?;
        let mut cmd = redis::cmd("SET")
            .arg(&[
                &cache_key,
                &body,
                "XX",
                "EX",
                &format!("{}", timeout.as_secs()),
            ])
            .clone();

        let value = self
            .execute_command(&mut cmd)
            .await
            .map(redis::Value::from)
            .map_err(StorageError::from)?;

        match value {
            redis::Value::Okay => Ok(()),
            redis::Value::Nil => Err(StorageError::BackendError(
                "Update returned nil response data".to_string(),
            )),
            _ => Err(StorageError::BackendError(
                "Update returned invalid response data".to_string(),
            )),
        }
    }

    async fn delete(&self, session_key: SessionKey) -> Result<(), Self::Error> {
        let key_gen = &self.config.key_gen;
        let cache_key = key_gen(&session_key);
        let mut cmd = redis::cmd("DEL").arg(&[&cache_key]).clone();

        self.execute_command(&mut cmd)
            .await
            .map_err(StorageError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_it() {
        RedisSessionStore::new("redis://localhost:6379")
            .await
            .unwrap();
    }
}
