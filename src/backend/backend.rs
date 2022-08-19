use std::time::Duration;

use crate::backend::session_key::SessionKey;
use crate::session::Session;

#[async_trait::async_trait(?Send)]
pub trait Backend {
    type Error;

    async fn load(&self, session_key: SessionKey) -> Result<Option<Session>, Self::Error>;
    async fn save(&self, session: Session, timeout: Duration) -> Result<SessionKey, Self::Error>;
    async fn update(
        &self,
        session_key: SessionKey,
        session: Session,
        timeout: Duration,
    ) -> Result<(), Self::Error>;
    async fn delete(&self, session_key: SessionKey) -> Result<(), Self::Error>;
}
