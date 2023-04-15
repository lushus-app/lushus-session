use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    storage::{Storage, StorageError},
    Session, SessionKey, Store,
};

pub struct SessionModel<'a, S: Store> {
    store: &'a S,
    session: Session,
    duration: Duration,
}

impl<'a, S: Store> SessionModel<'a, S> {
    pub fn new(store: &'a S, duration: Duration) -> Self {
        Self {
            store,
            duration,
            session: Default::default(),
        }
    }

    pub async fn load(
        store: &'a S,
        id: &SessionKey,
    ) -> Result<Option<SessionModel<'a, S>>, S::Error> {
        let session = store.load(id).await?;
        let duration = store.ttl(id).await?;
        let model = session.map(|session| Self {
            store,
            session,
            duration,
        });
        Ok(model)
    }

    pub fn id(&self) -> &SessionKey {
        self.session.id()
    }

    pub async fn save(&self) -> Result<(), S::Error> {
        let id = self.session.id();
        let exists = self.store.exists(id).await?;
        if exists {
            self.store.update(&self.session, self.duration).await
        } else {
            self.store.save(&self.session, self.duration).await
        }
    }

    pub async fn destroy(&self) -> Result<(), S::Error> {
        let id = self.session.id();
        self.store.destroy(id).await?;
        Ok(())
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn timeout(&self) -> Duration {
        self.duration
    }
}

impl<'a, S: Store> Storage<&str> for SessionModel<'a, S> {
    type Error = StorageError;

    fn insert<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Self::Error> {
        self.session.insert(key, value)
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        self.session.remove(key)
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        self.session.get(key)
    }
}

impl<'a, S: Store> From<SessionModel<'a, S>> for Session {
    fn from(model: SessionModel<'a, S>) -> Self {
        model.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{redis_store::RedisDatabase, storage::Storage};

    #[tokio::test]
    async fn save_commits_new_sessions_to_the_store() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::from_secs(1);
        let mut model = SessionModel::new(&store, timeout);
        model
            .insert("user_id", &"abc-123".to_string())
            .expect("Unable to insert user_id");
        model.save().await.expect("Unable to save session");

        let loaded_session = store
            .as_ref()
            .load(model.id())
            .await
            .expect("Unable to load session")
            .expect("Unable to find saved session");
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get from loaded session")
            .expect("Unable to find user_id in loaded session");
        assert_eq!(&loaded_user_id, "abc-123");
    }

    #[tokio::test]
    async fn save_updates_existing_session_in_the_store() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::from_secs(10);
        let mut model = SessionModel::new(&store, timeout);

        model
            .insert("user_id", &"beavis".to_string())
            .expect("Unable to insert user_id");
        model.save().await.expect("Unable to save session");

        let loaded_session = store
            .as_ref()
            .load(model.id())
            .await
            .expect("Unable to load session")
            .expect("Unable to find saved session");
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get from loaded session")
            .expect("Unable to find user_id in loaded session");
        assert_eq!(&loaded_user_id, "beavis");

        model
            .insert("user_id", &"butt-head".to_string())
            .expect("Unable to insert user_id");
        model.save().await.expect("Unable to save session");

        let loaded_session = store
            .as_ref()
            .load(model.id())
            .await
            .expect("Unable to load session")
            .expect("Unable to find saved session");
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get from loaded session")
            .expect("Unable to find user_id in loaded session");
        assert_eq!(&loaded_user_id, "butt-head");
    }

    #[tokio::test]
    async fn load_creates_a_model_from_existing_session_in_the_store() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let mut session = Session::default();
        session
            .insert("user_id", &"beavis".to_string())
            .expect("Unable to insert user_id");
        let timeout = Duration::from_secs(10);
        store
            .as_ref()
            .save(&session, timeout)
            .await
            .expect("Unable to save session");

        let model = SessionModel::load(&store, session.id())
            .await
            .expect("Unable to load session")
            .expect("Unable to find saved session");
        let user_id = model
            .get::<String>("user_id")
            .expect("Unable to get from loaded session")
            .expect("Unable to find user_id in loaded session");
        assert_eq!(&user_id, "beavis");
    }

    #[tokio::test]
    async fn destroy_deletes_the_session_to_the_store() {
        let store = RedisDatabase::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::from_secs(1);
        let mut model = SessionModel::new(&store, timeout);
        model
            .insert("user_id", &"abc-123".to_string())
            .expect("Unable to insert user_id");
        model.save().await.expect("Unable to save session");
        let exists = store
            .as_ref()
            .exists(model.id())
            .await
            .expect("Unable to check exists");
        assert!(exists);

        model.destroy().await.expect("Unable to destroy session");
        let exists = store
            .as_ref()
            .exists(model.id())
            .await
            .expect("Unable to check exists");
        assert!(!exists);
    }
}
