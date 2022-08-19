mod commands;

use redis::aio::ConnectionManager;
use std::time::Duration;

use crate::{
    session::Session,
    session_state::SessionState,
    session_store::{SessionKey, SessionStore},
};
use commands::Command;

struct Configuration {
    key_gen: Box<dyn Fn(&SessionKey) -> String + Send + Sync>,
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
pub enum StoreError {
    #[error("Storage error: Backend error {0}")]
    BackendError(String),
    #[error("Storage error: Unable to serialize or deserialize session from session key: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error(transparent)]
    RedisError(#[from] RedisError),
}

pub struct RedisSessionStore {
    config: Configuration,
    connection: ConnectionManager,
}

impl RedisSessionStore {
    pub async fn new(url: &str) -> Result<Self, RedisError> {
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
        command: Command,
    ) -> Result<T, RedisError> {
        let redis_command: redis::Cmd = command.into();
        let result = redis_command
            .query_async(&mut self.connection.clone())
            .await
            .map_err(|e| e.to_string())
            .map_err(RedisError::QueryError)?;
        Ok(result)
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for RedisSessionStore {
    type Error = StoreError;

    async fn load(&self, session_key: &SessionKey) -> Result<Option<Session>, Self::Error> {
        let cache_key = (&self.config.key_gen)(session_key);
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
        let cache_key = (&self.config.key_gen)(&session_id);
        let state: SessionState = session.into();
        let body = serde_json::to_string(&state).map_err(StoreError::SerializationError)?;
        self.execute_command(Command::set(cache_key, body, timeout))
            .await
            .map_err(StoreError::from)?;
        Ok(())
    }

    async fn update(&self, session: &Session, timeout: Duration) -> Result<(), Self::Error> {
        let session_id = session.id();
        let cache_key = (&self.config.key_gen)(&session_id);
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
        let cache_key = (&self.config.key_gen)(&session_key);
        self.execute_command(Command::delete(cache_key))
            .await
            .map_err(StoreError::from)
    }

    async fn exists(&self, session_key: &SessionKey) -> Result<bool, Self::Error> {
        let cache_key = (&self.config.key_gen)(&session_key);
        let exists = self
            .execute_command::<u64>(Command::exists(cache_key))
            .await
            .map_err(StoreError::from)?;
        Ok(exists > 0)
    }

    async fn ttl(&self, session_key: &SessionKey) -> Result<Duration, Self::Error> {
        let cache_key = (&self.config.key_gen)(&session_key);
        let ttl = self
            .execute_command::<u64>(Command::ttl(cache_key))
            .await
            .map_err(StoreError::from)?;
        Ok(Duration::from_secs(ttl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{session::Session, storage::Storage};

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
        let timeout = Duration::new(1, 0);
        store
            .save(&session, timeout)
            .await
            .expect("Unable to save session");

        let loaded_session = store.load(session.id()).await.unwrap().unwrap();
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get user id")
            .expect("User id not found");

        assert_eq!(loaded_user_id, user_id);
    }

    #[tokio::test]
    async fn update_overwrites_the_session_for_the_given_key() {
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");

        let timeout = Duration::new(1, 0);
        let user_id_1 = "beavis".to_string();
        let user_id_2 = "butt-head".to_string();

        let mut session = Session::default();
        session
            .insert("user_id", &user_id_1)
            .expect("Unable to insert user id");
        store
            .save(&session, timeout)
            .await
            .expect("Unable to save session");
        let loaded_session = store.load(session.id()).await.unwrap().unwrap();
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get user id")
            .expect("User id not found");
        assert_eq!(loaded_user_id, user_id_1);

        session
            .insert("user_id", &user_id_2)
            .expect("Unable to insert user id");
        store
            .update(&session, timeout)
            .await
            .expect("Unable to save session");
        let loaded_session = store.load(session.id()).await.unwrap().unwrap();
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get user id")
            .expect("User id not found");
        assert_eq!(loaded_user_id, user_id_2);
    }

    #[tokio::test]
    async fn exists_returns_true_if_the_given_session_key_is_stored() {
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::new(1, 0);
        let session = Session::default();
        store
            .save(&session, timeout)
            .await
            .expect("Unable to save session");
        let exists = store
            .exists(session.id())
            .await
            .expect("Unable to check exists");
        assert!(exists)
    }

    #[tokio::test]
    async fn exists_returns_false_if_the_given_session_key_is_not_stored() {
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let session = Session::default();
        let exists = store
            .exists(session.id())
            .await
            .expect("Unable to check exists");
        assert!(!exists)
    }
}
