use redis::aio::ConnectionManager;
use std::time::Duration;

use crate::backend::{Backend, SessionKey};
use crate::session::Session;
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

    async fn load(&self, session_key: SessionKey) -> Result<Option<Session>, Self::Error> {
        let cache_key = (&self.config.key_gen)(&session_key);
        let value = self
            .execute_command::<Option<String>>(redis::cmd("GET").arg(&[&cache_key]))
            .await
            .map_err(StorageError::from)?;
        value
            .map(|v| serde_json::from_str::<SessionState>(&v))
            .transpose()
            .map(|s| s.map(Session::from))
            .map_err(StorageError::SerializationError)
    }

    async fn save(&self, session: Session, timeout: Duration) -> Result<SessionKey, Self::Error> {
        let session_key = SessionKey::generate();
        let cache_key = (&self.config.key_gen)(&session_key);
        let state: SessionState = session.into();
        let body = serde_json::to_string(&state).map_err(StorageError::SerializationError)?;
        self.execute_command(redis::cmd("SET").arg(&[
            &cache_key,
            &body,
            "NX",
            "EX",
            &format!("{}", timeout.as_secs()),
        ]))
        .await
        .map_err(StorageError::from)?;

        Ok(session_key)
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session: Session,
        timeout: Duration,
    ) -> Result<(), Self::Error> {
        let cache_key = (&self.config.key_gen)(&session_key);
        let state: SessionState = session.into();
        let body = serde_json::to_string(&state).map_err(StorageError::SerializationError)?;
        let value = self
            .execute_command(redis::cmd("SET").arg(&[
                &cache_key,
                &body,
                "XX",
                "EX",
                &format!("{}", timeout.as_secs()),
            ]))
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
        let cache_key = (&self.config.key_gen)(&session_key);
        self.execute_command(redis::cmd("DEL").arg(&[&cache_key]))
            .await
            .map_err(StorageError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Session;

    #[tokio::test]
    async fn test_it() {
        RedisSessionStore::new("redis://localhost:6379")
            .await
            .expect("Unable to connect to Redis");
    }

    #[tokio::test]
    async fn load_returns_the_session_for_the_given_key() {
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");

        let user_id = "abc-123".to_string();

        let mut session = Session::default();
        session
            .insert("user_id", &user_id)
            .expect("Unable to insert user id");

        let session_key = store
            .save(session.into(), Duration::new(1, 0))
            .await
            .expect("Unable to save session");

        let loaded_session = store.load(session_key).await.unwrap().unwrap();
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get user id")
            .expect("User id not found");

        assert_eq!(loaded_user_id, user_id);
    }
}
