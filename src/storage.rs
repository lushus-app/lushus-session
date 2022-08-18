use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error(transparent)]
    StorageInsertError(#[from] StorageInsertError),
    #[error(transparent)]
    StorageGetError(#[from] StorageGetError),
    #[error(transparent)]
    StorageRemoveError(#[from] StorageRemoveError),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageInsertError {
    #[error("Unable to serialize value for key \"{0}\": {1}")]
    SerializeError(String, String),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageGetError {
    #[error("Unable to deserialize value for key \"{0}\": {1}")]
    DeserializeError(String, String),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageRemoveError {
    #[error("Unable to deserialize value for key \"{0}\": {1}")]
    DeserializeError(String, String),
}

pub trait Storage<K> {
    type Error;

    fn insert<T: Serialize>(&mut self, key: K, value: &T) -> Result<(), Self::Error>;
    fn remove<T: DeserializeOwned>(&mut self, key: K) -> Result<Option<T>, Self::Error>;
    fn get<T: DeserializeOwned>(&self, key: K) -> Result<Option<T>, Self::Error>;
    fn contains_key(&self, key: K) -> Result<bool, Self::Error>;
}
