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
    async fn ttl(&self, session_key: &SessionKey) -> Result<Duration, Self::Error>;
}

#[async_trait::async_trait(?Send)]
impl<S> SessionStore for &S
where
    S: SessionStore,
{
    type Error = S::Error;

    async fn load(&self, session_key: &SessionKey) -> Result<Option<Session>, Self::Error> {
        <S as SessionStore>::load(self, session_key).await
    }

    async fn save(&self, session: &Session, timeout: Duration) -> Result<(), Self::Error> {
        <S as SessionStore>::save(self, session, timeout).await
    }

    async fn update(&self, session: &Session, timeout: Duration) -> Result<(), Self::Error> {
        <S as SessionStore>::update(self, session, timeout).await
    }

    async fn destroy(&self, session_key: &SessionKey) -> Result<(), Self::Error> {
        <S as SessionStore>::destroy(self, session_key).await
    }

    async fn exists(&self, session_key: &SessionKey) -> Result<bool, Self::Error> {
        <S as SessionStore>::exists(self, session_key).await
    }

    async fn ttl(&self, session_key: &SessionKey) -> Result<Duration, Self::Error> {
        <S as SessionStore>::ttl(self, session_key).await
    }
}
