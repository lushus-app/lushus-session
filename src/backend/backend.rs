use std::time::Duration;

use crate::backend::session_key::SessionKey;
use crate::session_state::SessionState;

#[async_trait::async_trait(?Send)]
pub trait Backend {
    type Error;

    async fn load(&self, session_key: SessionKey) -> Result<Option<SessionState>, Self::Error>;
    async fn save(
        &self,
        session_state: SessionState,
        timeout: Duration,
    ) -> Result<SessionKey, Self::Error>;
    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        timeout: Duration,
    ) -> Result<(), Self::Error>;
    async fn delete(&self, session_key: SessionKey) -> Result<(), Self::Error>;
}
