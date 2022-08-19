use std::time::Duration;

use crate::{session::Session, session_store::session_key::SessionKey};

#[async_trait::async_trait(?Send)]
pub trait SessionStore {
    type Error;

    async fn load(&self, session_key: &SessionKey) -> Result<Option<Session>, Self::Error>;
    async fn save(&self, session: &Session, timeout: Duration) -> Result<(), Self::Error>;
    async fn update(&self, session: &Session, timeout: Duration) -> Result<(), Self::Error>;
    async fn destroy(&self, session_key: &SessionKey) -> Result<(), Self::Error>;
    async fn exists(&self, session_key: &SessionKey) -> Result<bool, Self::Error>;
}
