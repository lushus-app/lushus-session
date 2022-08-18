use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};

#[derive(Clone, Debug)]
pub struct SessionKey(String);

impl SessionKey {
    pub fn generate() -> Self {
        let value = std::iter::repeat(())
            .map(|()| OsRng.sample(Alphanumeric))
            .take(64)
            .collect::<Vec<_>>();
        let key = String::from_utf8(value).unwrap().try_into().unwrap();
        Self(key)
    }
}

impl AsRef<str> for SessionKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
