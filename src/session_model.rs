use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    session_storage::{SessionStorageError, SessionStorageRead, SessionStorageWrite},
    Session, SessionError, SessionKey,
};

pub struct SessionModel<S> {
    storage: S,
    session: Session,
    duration: Duration,
}

impl<S> SessionModel<S> {
    pub fn new(storage: S, duration: Duration) -> Self {
        Self {
            storage,
            duration,
            session: Default::default(),
        }
    }

    pub fn id(&self) -> &SessionKey {
        self.session.id()
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
        value: T,
    ) -> Result<Option<T>, SessionError> {
        self.session.insert(key, &value)
    }

    pub fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, SessionError> {
        self.session.remove(key)
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SessionError> {
        self.session.get(key)
    }
}

impl<S> SessionModel<S>
where
    S: SessionStorageRead,
{
    pub fn load(
        storage: S,
        id: &SessionKey,
    ) -> Result<Option<Self>, SessionStorageError<S::Error>> {
        let session = storage.session_load(id)?;
        let duration = storage.session_ttl(id)?;
        let model = session.map(|session| Self {
            storage,
            session,
            duration,
        });
        Ok(model)
    }
}

impl<S> SessionModel<S>
where
    S: SessionStorageWrite,
{
    pub fn save(&mut self) -> Result<(), SessionStorageError<S::Error>> {
        self.storage.session_save(&mut self.session)?;
        Ok(())
    }

    pub fn destroy(&mut self) -> Result<(), SessionStorageError<S::Error>> {
        let id = self.session.id();
        self.storage.session_destroy(id)?;
        Ok(())
    }
}

impl<S> From<SessionModel<S>> for Session {
    fn from(model: SessionModel<S>) -> Self {
        model.session
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, collections::HashMap, time::Duration};

    use lushus_storage::{Storage, StorageRead, StorageTemp, StorageWrite};

    use crate::{
        session_state::SessionState, session_storage::SessionStateTable, SessionKey, SessionModel,
    };

    struct TestStorage {
        map: HashMap<SessionKey, SessionState>,
    }

    impl TestStorage {
        fn new() -> Self {
            let map = HashMap::new();
            TestStorage { map }
        }
    }

    impl Storage for TestStorage {
        type Error = std::convert::Infallible;
    }

    impl StorageRead<SessionStateTable> for TestStorage {
        fn get(&self, key: &SessionKey) -> Result<Option<Cow<'_, SessionState>>, Self::Error> {
            let result = self.map.get(key);
            let value = result.map(Cow::Borrowed);
            Ok(value)
        }

        fn exists(&self, key: &SessionKey) -> Result<bool, Self::Error> {
            let result = self.map.get(key);
            Ok(result.is_some())
        }
    }

    impl StorageWrite<SessionStateTable> for TestStorage {
        fn insert(
            &mut self,
            key: &SessionKey,
            value: &SessionState,
        ) -> Result<Option<SessionState>, Self::Error> {
            let previous = self.map.insert(key.clone(), value.clone());
            Ok(previous)
        }

        fn remove(&mut self, key: &SessionKey) -> Result<Option<SessionState>, Self::Error> {
            let previous = self.map.remove(key);
            Ok(previous)
        }
    }

    impl StorageTemp<SessionStateTable> for TestStorage {
        fn ttl(&self, _key: &SessionKey) -> Result<Duration, Self::Error> {
            Ok(Duration::from_secs(100))
        }
    }

    #[test]
    fn save_inserts_the_session() {
        let mut storage = TestStorage::new();

        let mut model = SessionModel::new(&mut storage, Duration::from_secs(100));
        model
            .insert::<String>("id", "abc".to_string())
            .expect("failed to write to session model");
        model.save().expect("Failed to save session model");
        let key = model.id().clone();

        let state = storage
            .get(&key)
            .expect("Failed to retrieve state from storage")
            .expect("Expected state to be present");
        let id = state.get("id").expect("Expected id to be present");
        assert_eq!(id, "\"abc\"");
    }

    #[test]
    fn load_retrieves_the_session() {
        let mut storage = TestStorage::new();
        let mut model = SessionModel::new(&mut storage, Duration::from_secs(100));
        model
            .insert::<String>("id", "abc".to_string())
            .expect("Failed write to session model");
        model.save().expect("Failed to save session model");
        let id = model.id().clone();

        let model = SessionModel::load(&mut storage, &id)
            .expect("Failed to load session model")
            .expect("Expected session model to be present");

        let id = model
            .get::<String>("id")
            .expect("Failed to read from session model")
            .expect("Expected id to be present");
        assert_eq!(id, "abc".to_string())
    }

    #[test]
    fn delete_removes_the_session() {
        let mut storage = TestStorage::new();
        let key = SessionKey::generate();
        let mut state = SessionState::default();
        state.insert("id", "abc".to_string());
        storage
            .insert(&key, &state)
            .expect("Failed to insert session state");

        storage
            .remove(&key)
            .expect("Failed to remove session state");
        let retrieved = storage.get(&key).expect("Failed to get session state");
        assert!(retrieved.is_none())
    }
}
