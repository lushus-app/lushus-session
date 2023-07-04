use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};

use crate::{session_store::StoreError, Session, SessionError, SessionKey, SessionStore};

pub struct SessionModel<'a, S: SessionStore> {
    store: &'a mut S,
    session: Session,
    duration: Duration,
}

impl<'a, S: SessionStore> SessionModel<'a, S> {
    pub fn new(store: &'a mut S, duration: Duration) -> Self {
        Self {
            store,
            duration,
            session: Default::default(),
        }
    }

    pub fn load(
        store: &'a mut S,
        id: &SessionKey,
    ) -> Result<Option<SessionModel<'a, S>>, StoreError<S::Error>> {
        let session = store.load(id)?;
        let duration = store.ttl(id)?;
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

    pub fn save(&mut self) -> Result<(), StoreError<S::Error>> {
        let id = self.session.id();
        let exists = self.store.exists(id)?;
        if exists {
            self.store.update(&self.session)?;
        } else {
            self.store.save(&self.session)?;
        }
        Ok(())
    }

    pub fn destroy(&mut self) -> Result<(), StoreError<S::Error>> {
        let id = self.session.id();
        self.store.destroy(id)?;
        Ok(())
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn timeout(&self) -> Duration {
        self.duration
    }

    pub fn insert<T: Serialize + DeserializeOwned>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<Option<T>, SessionError> {
        self.session.insert(key, value)
    }

    pub fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, SessionError> {
        self.session.remove(key)
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SessionError> {
        self.session.get(key)
    }
}

impl<'a, S: SessionStore> From<SessionModel<'a, S>> for Session {
    fn from(model: SessionModel<'a, S>) -> Self {
        model.session
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn save_commits_new_sessions_to_the_store() {
//         let store = RedisDatabase::new("redis://:password@localhost:6379/1")
//             .expect("Unable to connect to Redis");
//         let timeout = Duration::from_secs(1);
//         let mut model = SessionModel::new(&store, timeout);
//         model
//             .insert("user_id", &"abc-123".to_string())
//             .expect("Unable to insert user_id");
//         model.save().expect("Unable to save session");
//
//         let loaded_session = store
//             .as_ref()
//             .load(model.id())
//             .expect("Unable to load session")
//             .expect("Unable to find saved session");
//         let loaded_user_id = loaded_session
//             .get::<String>("user_id")
//             .expect("Unable to get from loaded session")
//             .expect("Unable to find user_id in loaded session");
//         assert_eq!(&loaded_user_id, "abc-123");
//     }
//
//     #[tokio::test]
//     async fn save_updates_existing_session_in_the_store() {
//         let store = RedisDatabase::new("redis://:password@localhost:6379/1")
//             .expect("Unable to connect to Redis");
//         let timeout = Duration::from_secs(10);
//         let mut model = SessionModel::new(&store, timeout);
//
//         model
//             .insert("user_id", &"beavis".to_string())
//             .expect("Unable to insert user_id");
//         model.save().expect("Unable to save session");
//
//         let loaded_session = store
//             .as_ref()
//             .load(model.id())
//             .expect("Unable to load session")
//             .expect("Unable to find saved session");
//         let loaded_user_id = loaded_session
//             .get::<String>("user_id")
//             .expect("Unable to get from loaded session")
//             .expect("Unable to find user_id in loaded session");
//         assert_eq!(&loaded_user_id, "beavis");
//
//         model
//             .insert("user_id", &"butt-head".to_string())
//             .expect("Unable to insert user_id");
//         model.save().expect("Unable to save session");
//
//         let loaded_session = store
//             .as_ref()
//             .load(model.id())
//             .expect("Unable to load session")
//             .expect("Unable to find saved session");
//         let loaded_user_id = loaded_session
//             .get::<String>("user_id")
//             .expect("Unable to get from loaded session")
//             .expect("Unable to find user_id in loaded session");
//         assert_eq!(&loaded_user_id, "butt-head");
//     }
//
//     #[tokio::test]
//     async fn load_creates_a_model_from_existing_session_in_the_store() {
//         let store = RedisDatabase::new("redis://:password@localhost:6379/1")
//             .expect("Unable to connect to Redis");
//         let mut session = Session::default();
//         session
//             .insert("user_id", &"beavis".to_string())
//             .expect("Unable to insert user_id");
//         let timeout = Duration::from_secs(10);
//         store
//             .as_ref()
//             .save(&session, timeout)
//             .expect("Unable to save session");
//
//         let model = SessionModel::load(&store, session.id())
//             .expect("Unable to load session")
//             .expect("Unable to find saved session");
//         let user_id = model
//             .get::<String>("user_id")
//             .expect("Unable to get from loaded session")
//             .expect("Unable to find user_id in loaded session");
//         assert_eq!(&user_id, "beavis");
//     }
//
//     #[tokio::test]
//     async fn destroy_deletes_the_session_to_the_store() {
//         let store = RedisDatabase::new("redis://:password@localhost:6379/1")
//             .expect("Unable to connect to Redis");
//         let timeout = Duration::from_secs(1);
//         let mut model = SessionModel::new(&store, timeout);
//         model
//             .insert("user_id", &"abc-123".to_string())
//             .expect("Unable to insert user_id");
//         model.save().expect("Unable to save session");
//         let exists = store
//             .as_ref()
//             .exists(model.id())
//             .expect("Unable to check exists");
//         assert!(exists);
//
//         model.destroy().expect("Unable to destroy session");
//         let exists = store
//             .as_ref()
//             .exists(model.id())
//             .expect("Unable to check exists");
//         assert!(!exists);
//     }
// }
