use serde::{de::DeserializeOwned, Serialize};

use crate::{session_state::SessionState, SessionKey};

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session is destroyed")]
    SessionDestroyedError,
    #[error("Unable to serialize key \"{0}\" value {1}")]
    SerializationError(String, String),
    #[error("Unable to deserialize key \"{0}\" value {1}")]
    DeserializationError(String, String),
}

#[derive(Default)]
pub struct Session {
    id: SessionKey,
    state: SessionState,
}

impl Session {
    pub fn new(id: SessionKey, state: SessionState) -> Self {
        Session { id, state }
    }

    pub fn id(&self) -> &SessionKey {
        &self.id
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn insert<T: Serialize + DeserializeOwned>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<Option<T>, SessionError> {
        let insert = serde_json::to_string(value)
            .map_err(|e| SessionError::SerializationError(key.to_string(), e.to_string()))?;
        let previous = self
            .state
            .insert(key, insert)
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|e| SessionError::DeserializationError(key.to_string(), e.to_string()));
        previous
    }

    pub fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, SessionError> {
        self.state
            .remove(key)
            .map(|v| serde_json::from_str(&v))
            .transpose()
            .map_err(|e| SessionError::DeserializationError(key.to_string(), e.to_string()))
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SessionError> {
        self.state
            .get(key)
            .map(|v| serde_json::from_str(v))
            .transpose()
            .map_err(|e| SessionError::DeserializationError(key.to_string(), e.to_string()))
    }
}

impl From<Session> for SessionState {
    fn from(session: Session) -> Self {
        session.state
    }
}

impl From<&Session> for SessionState {
    fn from(session: &Session) -> Self {
        session.state().clone()
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct User {
        username: String,
        password: String,
    }

    #[test]
    fn insert_inserts_the_given_key_and_value() {
        let mut session = Session::default();
        let user = User {
            username: "brandon".to_string(),
            password: "hunter2".to_string(),
        };
        session
            .insert("user", &user)
            .expect("unable to insert User");
    }

    #[test]
    fn remove_removes_the_value_for_the_given_key() {
        let mut session = Session::default();
        let user = User {
            username: "brandon".to_string(),
            password: "hunter2".to_string(),
        };
        session
            .insert("user", &user)
            .expect("expected insert \"user\" to insert User");

        session
            .remove::<User>("user")
            .expect("expected remove \"user\" to succeed")
            .expect("expected remove \"user\" to return a User");

        let user = session
            .get::<User>("user")
            .expect("expected get \"user\" to succeed");
        assert_eq!(user, None, "expected get \"user\" to return None");
    }

    #[test]
    fn get_returns_the_expected_value_for_the_given_key() {
        let mut session = Session::default();
        let user = User {
            username: "brandon".to_string(),
            password: "hunter2".to_string(),
        };
        session
            .insert("user", &user)
            .expect("unable to insert User");

        let user = session
            .get::<User>("user")
            .expect("expected get \"user\" to succeed")
            .expect("expected get \"user\" to return a User");
        assert_eq!(user.username, "brandon".to_string());
        assert_eq!(user.password, "hunter2".to_string());
    }
}
