use redis::aio::ConnectionManager;

use crate::{
    redis_session::{
        commands::Command, connect::Connect, error::RedisError, execute_command::ExecuteCommand,
    },
    session_store::configuration::Configuration,
};

pub struct RedisDatabase {
    config: Configuration,
    connection: ConnectionManager,
}

impl RedisDatabase {
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
}

impl AsRef<RedisDatabase> for RedisDatabase {
    fn as_ref(&self) -> &RedisDatabase {
        self
    }
}

impl Connect for RedisDatabase {
    fn config(&self) -> &Configuration {
        &self.config
    }

    fn connection(&self) -> &ConnectionManager {
        &self.connection
    }
}

#[async_trait::async_trait(?Send)]
impl ExecuteCommand for RedisDatabase {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{session::Session, storage::Storage, Store};

    #[tokio::test]
    async fn test_it() {
        RedisDatabase::new("redis://localhost:6379")
            .await
            .expect("Unable to connect to Redis");
    }

    #[tokio::test]
    async fn load_returns_the_session_for_the_given_key() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");

        let user_id = "abc-123".to_string();

        let mut session = Session::default();
        session
            .insert("user_id", &user_id)
            .expect("Unable to insert user id");
        let timeout = Duration::new(1, 0);
        store
            .as_ref()
            .save(&session, timeout)
            .await
            .expect("Unable to save session");

        let loaded_session = store.as_ref().load(session.id()).await.unwrap().unwrap();
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get user id")
            .expect("User id not found");

        assert_eq!(loaded_user_id, user_id);
    }

    #[tokio::test]
    async fn update_overwrites_the_session_for_the_given_key() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
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
            .as_ref()
            .save(&session, timeout)
            .await
            .expect("Unable to save session");
        let loaded_session = store.as_ref().load(session.id()).await.unwrap().unwrap();
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get user id")
            .expect("User id not found");
        assert_eq!(loaded_user_id, user_id_1);

        session
            .insert("user_id", &user_id_2)
            .expect("Unable to insert user id");
        store
            .as_ref()
            .update(&session, timeout)
            .await
            .expect("Unable to save session");
        let loaded_session = store.as_ref().load(session.id()).await.unwrap().unwrap();
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get user id")
            .expect("User id not found");
        assert_eq!(loaded_user_id, user_id_2);
    }

    #[tokio::test]
    async fn exists_returns_true_if_the_given_session_key_is_stored() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::new(1, 0);
        let session = Session::default();
        store
            .as_ref()
            .save(&session, timeout)
            .await
            .expect("Unable to save session");
        let exists = store
            .as_ref()
            .exists(session.id())
            .await
            .expect("Unable to check exists");
        assert!(exists)
    }

    #[tokio::test]
    async fn exists_returns_false_if_the_given_session_key_is_not_stored() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let session = Session::default();
        let exists = store
            .as_ref()
            .exists(session.id())
            .await
            .expect("Unable to check exists");
        assert!(!exists)
    }
}
