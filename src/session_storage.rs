use std::time::Duration;

use lushus_storage::{Storage, StorageRead, StorageTemp, StorageWrite, Table};

use crate::{session::Session, session_state::SessionState, SessionKey};

#[derive(Debug, thiserror::Error)]
pub enum SessionStorageError<StorageError> {
    #[error("Serialization error")]
    SerializationError,
    #[error(transparent)]
    StorageError(#[from] StorageError),
}

pub struct SessionStateTable {}

impl Table for SessionStateTable {
    type Key = SessionKey;
    type OwnedKey = Self::Key;
    type Value = SessionState;
    type OwnedValue = Self::Value;
}

pub trait SessionStorageRead
where
    Self: Storage,
{
    fn session_exists(
        &self,
        session_key: &SessionKey,
    ) -> Result<bool, SessionStorageError<Self::Error>>;
    fn session_load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<Session>, SessionStorageError<Self::Error>>;
    fn session_ttl(
        &self,
        session_key: &SessionKey,
    ) -> Result<Duration, SessionStorageError<Self::Error>>;
}

pub trait SessionStorageWrite
where
    Self: Storage,
{
    fn session_save(&mut self, session: &Session) -> Result<(), SessionStorageError<Self::Error>>;
    fn session_destroy(
        &mut self,
        session_key: &SessionKey,
    ) -> Result<(), SessionStorageError<Self::Error>>;
}

impl<S> SessionStorageRead for S
where
    S: StorageRead<SessionStateTable> + StorageTemp<SessionStateTable>,
{
    fn session_exists(
        &self,
        session_key: &SessionKey,
    ) -> Result<bool, SessionStorageError<Self::Error>> {
        let exists = self.exists(session_key)?;
        Ok(exists)
    }

    fn session_load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<Session>, SessionStorageError<Self::Error>> {
        let state = self.get(session_key)?;
        let session = state.map(|state| Session::new(session_key.clone(), state.into_owned()));
        Ok(session)
    }

    fn session_ttl(
        &self,
        session_key: &SessionKey,
    ) -> Result<Duration, SessionStorageError<Self::Error>> {
        let ttl = self.ttl(session_key)?;
        Ok(ttl)
    }
}

impl<S> SessionStorageWrite for S
where
    S: StorageWrite<SessionStateTable>,
{
    fn session_save(&mut self, session: &Session) -> Result<(), SessionStorageError<Self::Error>> {
        let session_id = session.id();
        let state: SessionState = session.into();
        self.insert(session_id, &state)?;
        Ok(())
    }

    fn session_destroy(
        &mut self,
        session_key: &SessionKey,
    ) -> Result<(), SessionStorageError<Self::Error>> {
        self.remove(session_key)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, collections::HashMap};

    use lushus_storage::{Storage, StorageRead, StorageWrite};

    use crate::{session_state::SessionState, session_storage::SessionStateTable, SessionKey};

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

    #[test]
    fn insert_inserts_the_session_state() {
        let mut storage = TestStorage::new();
        let key = SessionKey::generate();
        let mut state = SessionState::default();
        state.insert("id", "abc".to_string());
        storage
            .insert(&key, &state)
            .expect("Failed to insert session state");
    }

    #[test]
    fn get_retrieves_the_session_state() {
        let mut storage = TestStorage::new();
        let key = SessionKey::generate();
        let mut state = SessionState::default();
        state.insert("id", "abc".to_string());
        storage
            .insert(&key, &state)
            .expect("Failed to insert session state");

        let retrieved = storage
            .get(&key)
            .expect("Failed to get session state")
            .expect("Expected session state to be present");
        assert_eq!(Cow::Borrowed(&state), retrieved)
    }

    #[test]
    fn remove_removes_the_session_state() {
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
