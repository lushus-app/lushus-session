use std::fmt::{Display, Formatter};

use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionKey(String);

impl Display for SessionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SessionKey {
    pub fn generate() -> Self {
        let value = std::iter::repeat(())
            .map(|()| OsRng.sample(Alphanumeric))
            .take(64)
            .collect::<Vec<_>>();
        let key = String::from_utf8(value).unwrap();
        Self(key)
    }
}

impl AsRef<str> for SessionKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for SessionKey {
    fn default() -> Self {
        Self::generate()
    }
}
