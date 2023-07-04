use std::time::Duration;

use lushus_storage::storage::{StorageRead, StorageTemp, StorageWrite};

use crate::{session::Session, session_state::SessionState, SessionKey};

#[derive(Debug, thiserror::Error)]
pub enum StoreError<S> {
    #[error("Serialization error")]
    SerializationError,
    #[error("Expected key {0} to be present")]
    UpdateError(String),
    #[error(transparent)]
    StorageError(#[from] S),
}

pub trait SessionStore
where
    Self: StorageRead<SessionKey> + StorageWrite<SessionKey>,
{
    fn load(&self, session_key: &SessionKey) -> Result<Option<Session>, StoreError<Self::Error>>;
    fn save(&mut self, session: &Session) -> Result<(), StoreError<Self::Error>>;
    fn update(&mut self, session: &Session) -> Result<(), StoreError<Self::Error>>;
    fn destroy(&mut self, session_key: &SessionKey) -> Result<(), StoreError<Self::Error>>;
    fn ttl(&self, session_key: &SessionKey) -> Result<Duration, StoreError<Self::Error>>;
}

impl<S> SessionStore for S
where
    S: StorageRead<SessionKey> + StorageWrite<SessionKey> + StorageTemp<SessionKey>,
{
    fn load(&self, session_key: &SessionKey) -> Result<Option<Session>, StoreError<Self::Error>> {
        let state = self.get::<SessionState>(session_key)?;
        let session = state.map(|state| Session::new(session_key.clone(), state));
        Ok(session)
    }

    fn save(&mut self, session: &Session) -> Result<(), StoreError<Self::Error>> {
        let session_id = session.id();
        let state: SessionState = session.into();
        let body = serde_json::to_string(&state).map_err(|_| StoreError::SerializationError)?;
        self.insert(session_id, &body).map_err(StoreError::from)?;
        Ok(())
    }

    fn update(&mut self, session: &Session) -> Result<(), StoreError<Self::Error>> {
        let session_id = session.id();
        let state: SessionState = session.into();
        let body = serde_json::to_string(&state).map_err(|_| StoreError::SerializationError)?;
        let prev = self.insert(session_id, &body).map_err(StoreError::from)?;
        match prev {
            Some(_) => Ok(()),
            None => Err(StoreError::UpdateError(session_id.to_string())),
        }
    }

    fn destroy(&mut self, session_key: &SessionKey) -> Result<(), StoreError<Self::Error>> {
        self.remove::<SessionState>(session_key)?;
        Ok(())
    }

    fn ttl(&self, session_key: &SessionKey) -> Result<Duration, StoreError<Self::Error>> {
        let ttl = self.ttl(session_key).map_err(StoreError::from)?;
        Ok(ttl)
    }
}
