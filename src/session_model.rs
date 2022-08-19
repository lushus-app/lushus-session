use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::{
    storage::{Storage, StorageError},
    Session, SessionKey, SessionStore,
};

pub struct SessionModel<Store: SessionStore> {
    store: Store,
    session: Session,
    duration: Duration,
}

impl<Store: SessionStore> SessionModel<Store> {
    pub fn new(store: Store, duration: Duration) -> Self {
        Self {
            store,
            duration,
            session: Default::default(),
        }
    }

    pub async fn load(
        store: Store,
        id: &SessionKey,
        duration: Duration,
    ) -> Result<Option<Self>, Store::Error> {
        let session = store.load(id).await?;
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

    pub async fn save(&self) -> Result<(), Store::Error> {
        let id = self.session.id();
        let exists = self.store.exists(id).await?;
        if exists {
            self.store.update(&self.session, self.duration).await
        } else {
            self.store.save(&self.session, self.duration).await
        }
    }

    pub async fn destroy(&self) -> Result<(), Store::Error> {
        let id = self.session.id();
        self.store.destroy(id).await?;
        Ok(())
    }
}

impl<Store: SessionStore> Storage<&str> for SessionModel<Store> {
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

impl<Store: SessionStore> From<SessionModel<Store>> for Session {
    fn from(model: SessionModel<Store>) -> Self {
        model.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::Storage, RedisSessionStore};

    #[tokio::test]
    async fn save_commits_new_sessions_to_the_store() {
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::from_secs(1);
        let mut model = SessionModel::new(&store, timeout);
        model
            .insert("user_id", &"abc-123".to_string())
            .expect("Unable to insert user_id");
        model.save().await.expect("Unable to save session");

        let loaded_session = store
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
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::from_secs(10);
        let mut model = SessionModel::new(&store, timeout);

        model
            .insert("user_id", &"beavis".to_string())
            .expect("Unable to insert user_id");
        model.save().await.expect("Unable to save session");

        let loaded_session = store
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
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let mut session = Session::default();
        session
            .insert("user_id", &"beavis".to_string())
            .expect("Unable to insert user_id");
        let timeout = Duration::from_secs(10);
        store
            .save(&session, timeout)
            .await
            .expect("Unable to save session");

        let model = SessionModel::load(&store, session.id(), timeout)
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
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let timeout = Duration::from_secs(1);
        let mut model = SessionModel::new(&store, timeout);
        model
            .insert("user_id", &"abc-123".to_string())
            .expect("Unable to insert user_id");
        model.save().await.expect("Unable to save session");
        let exists = store
            .exists(model.id())
            .await
            .expect("Unable to check exists");
        assert!(exists);

        model.destroy().await.expect("Unable to destroy session");
        let exists = store
            .exists(model.id())
            .await
            .expect("Unable to check exists");
        assert!(!exists);
    }
}
