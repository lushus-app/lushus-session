use crate::SessionKey;

pub trait GenerateKey {
    fn generate_key(&self, session_key: &SessionKey) -> String;
}
