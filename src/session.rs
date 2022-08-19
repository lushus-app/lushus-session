use serde::{de::DeserializeOwned, Serialize};
use std::borrow::{Borrow, BorrowMut};

use crate::{
    session_state::SessionState,
    storage::{Storage, StorageError, StorageGetError, StorageInsertError, StorageRemoveError},
    SessionKey,
};

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    SessionStorageError(#[from] StorageError),
    #[error("Session is destroyed")]
    SessionDestroyedError,
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

    // pub fn status(&self) -> SessionStatus {
    //     self.status
    // }
    //
    // pub fn destroy(&mut self) {
    //     self.status = SessionStatus::Destroyed;
    // }
    //
    // pub fn renew(&mut self) {
    //     self.status = SessionStatus::Renewed;
    // }
    //
    // pub fn active(&self) -> bool {
    //     self.status != SessionStatus::Destroyed
    // }

    // pub fn insert<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), SessionError> {
    //     if self.active() {
    //         let result = <Self as Storage<&str>>::insert(self, key, value)?;
    //         self.status = SessionStatus::Changed;
    //         Ok(result)
    //     } else {
    //         Err(SessionError::SessionDestroyedError)
    //     }
    // }
    //
    // pub fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, SessionError> {
    //     if self.active() {
    //         let result = <Self as Storage<&str>>::remove(self, key)?;
    //         self.status = SessionStatus::Changed;
    //         Ok(result)
    //     } else {
    //         Err(SessionError::SessionDestroyedError)
    //     }
    // }
    //
    // pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, SessionError> {
    //     if self.active() {
    //         let result = <Self as Storage<&str>>::get(self, key)?;
    //         Ok(result)
    //     } else {
    //         Err(SessionError::SessionDestroyedError)
    //     }
    // }
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

// impl From<SessionState> for Session {
//     fn from(state: SessionState) -> Self {
//         Self {
//             state,
//             ..Default::default()
//         }
//     }
// }

impl Storage<&str> for Session {
    type Error = StorageError;

    fn insert<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Self::Error> {
        let insert = serde_json::to_string(value)
            .map_err(|e| StorageInsertError::SerializeError(key.to_string(), e.to_string()))
            .map_err(StorageError::from)?;
        self.state.borrow_mut().insert(key, insert);
        Ok(())
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        self.state
            .borrow_mut()
            .remove(key)
            .map(|v| serde_json::from_str(&v))
            .transpose()
            .map_err(|e| StorageRemoveError::DeserializeError(key.to_string(), e.to_string()))
            .map_err(StorageError::from)
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        self.state
            .borrow()
            .get(key)
            .map(|v| serde_json::from_str(v))
            .transpose()
            .map_err(|e| StorageGetError::DeserializeError(key.to_string(), e.to_string()))
            .map_err(StorageError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct User {
        username: String,
        password: String,
    }

    // #[test]
    // fn status_returns_unchanged_by_default() {
    //     let session = Session::default();
    //     let status = session.status();
    //     assert_eq!(status, SessionStatus::Unchanged);
    // }
    //
    // #[test]
    // fn status_returns_changed_after_insert() {
    //     let mut session = Session::default();
    //     let user = User {
    //         username: "brandon".to_string(),
    //         password: "hunter2".to_string(),
    //     };
    //     session
    //         .insert("user", &user)
    //         .expect("unable to insert User");
    //
    //     let status = session.status();
    //     assert_eq!(status, SessionStatus::Changed);
    // }
    //
    // #[test]
    // fn status_returns_changed_after_remove() {
    //     let mut session = Session::default();
    //     session
    //         .remove::<User>("user")
    //         .expect("unable to remove User");
    //
    //     // We expect a status of `Changed` even if the remove operation returned no matching object.
    //     let status = session.status();
    //     assert_eq!(status, SessionStatus::Changed);
    // }
    //
    // #[test]
    // fn status_returns_destroyed_after_destroy() {
    //     let mut session = Session::default();
    //     session.destroy();
    //
    //     // We expect a status of `Changed` even if the remove operation returned no matching object.
    //     let status = session.status();
    //     assert_eq!(status, SessionStatus::Destroyed);
    // }

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

    // #[test]
    // fn insert_returns_session_destroyed_error_when_the_session_is_marked_destroyed() {
    //     let mut session = Session::default();
    //     let user = User {
    //         username: "brandon".to_string(),
    //         password: "hunter2".to_string(),
    //     };
    //     session
    //         .insert("user", &user)
    //         .expect("unable to insert User");
    //
    //     session.destroy();
    //
    //     let err = session
    //         .insert("user", &user)
    //         .expect_err("expected session to be destroyed");
    //     assert!(matches!(
    //         <dyn std::error::Error>::downcast_ref::<SessionError>(&err),
    //         Some(SessionError::SessionDestroyedError)
    //     ));
    // }
    //
    // #[test]
    // fn remove_returns_session_destroyed_error_when_the_session_is_marked_destroyed() {
    //     let mut session = Session::default();
    //     let user = User {
    //         username: "brandon".to_string(),
    //         password: "hunter2".to_string(),
    //     };
    //     session
    //         .insert("user", &user)
    //         .expect("unable to insert User");
    //
    //     session.destroy();
    //
    //     let err = session
    //         .remove::<User>("user")
    //         .expect_err("expected session to be destroyed");
    //     assert!(matches!(
    //         <dyn std::error::Error>::downcast_ref::<SessionError>(&err),
    //         Some(SessionError::SessionDestroyedError)
    //     ));
    // }
    //
    // #[test]
    // fn get_returns_session_destroyed_error_when_the_session_is_marked_destroyed() {
    //     let mut session = Session::default();
    //     let user = User {
    //         username: "brandon".to_string(),
    //         password: "hunter2".to_string(),
    //     };
    //     session
    //         .insert("user", &user)
    //         .expect("unable to insert User");
    //
    //     session.destroy();
    //
    //     let err = session
    //         .get::<User>("user")
    //         .expect_err("expected session to be destroyed");
    //     assert!(matches!(
    //         <dyn std::error::Error>::downcast_ref::<SessionError>(&err),
    //         Some(SessionError::SessionDestroyedError)
    //     ));
    // }
}
