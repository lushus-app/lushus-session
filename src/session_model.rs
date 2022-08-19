use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::{
    storage::{Storage, StorageError},
    Session, SessionKey, SessionStore,
};

struct SessionModel<Store: SessionStore> {
    store: Store,
    session: Session,
    duration: Duration,
}

impl<Store: SessionStore> SessionModel<Store> {
    pub fn new(store: Store, session: Session, duration: Duration) -> Self {
        Self {
            store,
            session,
            duration,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use crate::RedisSessionStore;

    #[tokio::test]
    async fn save_commits_new_sessions_to_the_store() {
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let mut session = Session::default();
        session
            .insert("user_id", &"abc-123".to_string())
            .expect("Unable to insert user_id");
        let timeout = Duration::from_secs(1);
        let session_model = SessionModel::new(&store, session, timeout);
        session_model.save().await.expect("Unable to save session");

        let loaded_session = store
            .load(session_model.id())
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
        let session = Session::default();
        let timeout = Duration::from_secs(10);
        let mut session_model = SessionModel::new(&store, session, timeout);

        session_model
            .insert("user_id", &"beavis".to_string())
            .expect("Unable to insert user_id");
        session_model.save().await.expect("Unable to save session");

        let loaded_session = store
            .load(session_model.id())
            .await
            .expect("Unable to load session")
            .expect("Unable to find saved session");
        let loaded_user_id = loaded_session
            .get::<String>("user_id")
            .expect("Unable to get from loaded session")
            .expect("Unable to find user_id in loaded session");
        assert_eq!(&loaded_user_id, "beavis");

        session_model
            .insert("user_id", &"butt-head".to_string())
            .expect("Unable to insert user_id");
        session_model.save().await.expect("Unable to save session");

        let loaded_session = store
            .load(session_model.id())
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
    async fn destroy_deletes_the_session_to_the_store() {
        let store = RedisSessionStore::new("redis://:password@localhost:6379/1")
            .await
            .expect("Unable to connect to Redis");
        let mut session = Session::default();
        session
            .insert("user_id", &"abc-123".to_string())
            .expect("Unable to insert user_id");
        let timeout = Duration::from_secs(1);
        let session_model = SessionModel::new(&store, session, timeout);

        session_model.save().await.expect("Unable to save session");
        let exists = store
            .exists(session_model.id())
            .await
            .expect("Unable to check exists");
        assert!(exists);
        session_model
            .destroy()
            .await
            .expect("Unable to destroy session");
        let exists = store
            .exists(session_model.id())
            .await
            .expect("Unable to check exists");
        assert!(!exists);
    }
}
