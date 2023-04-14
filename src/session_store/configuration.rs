use crate::SessionKey;

type KeyGen = dyn Fn(&SessionKey) -> String + Send + Sync;

pub struct Configuration {
    key_gen: Box<KeyGen>,
}

impl Configuration {
    pub fn key_gen(&self) -> &KeyGen {
        &self.key_gen
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            key_gen: Box::new(|v| v.as_ref().to_owned()),
        }
    }
}
