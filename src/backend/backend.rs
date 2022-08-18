use std::time::Duration;

use crate::backend::session_key::SessionKey;
use crate::session_state::SessionState;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {}

pub trait Backend {
    fn load(&self, session_key: SessionKey) -> Result<Option<SessionState>, StorageError>;

    fn save(
        &self,
        session_state: SessionState,
        timeout: Duration,
    ) -> Result<SessionKey, StorageError>;

    fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
    ) -> Result<SessionKey, StorageError>;

    fn delete(&self, session_key: SessionKey) -> Result<(), StorageError>;
}
