use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SessionState(HashMap<String, String>);

impl SessionState {
    pub fn insert(&mut self, key: &str, value: String) -> Option<String> {
        self.0.insert(key.to_string(), value)
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}
