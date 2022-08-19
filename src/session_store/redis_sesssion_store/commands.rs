use std::time::Duration;

pub enum Command {
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
        ttl: Duration,
    },
    Update {
        key: String,
        value: String,
        ttl: Duration,
    },
    Delete {
        key: String,
    },
}

impl Command {
    pub fn get(key: String) -> Self {
        Self::Get { key }
    }

    pub fn set(key: String, value: String, ttl: Duration) -> Self {
        Self::Set { key, value, ttl }
    }

    pub fn update(key: String, value: String, ttl: Duration) -> Self {
        Self::Update { key, value, ttl }
    }

    pub fn delete(key: String) -> Self {
        Self::Delete { key }
    }
}

impl From<Command> for redis::Cmd {
    fn from(command: Command) -> Self {
        match command {
            Command::Get { key } => redis::cmd("GET").arg(&[&key]).clone(),
            Command::Set { key, value, ttl } => redis::cmd("SET")
                .arg(&[
                    &key,
                    &value,
                    "NX",
                    "EX",
                    format!("{}", ttl.as_secs()).as_ref(),
                ])
                .clone(),
            Command::Update { key, value, ttl } => redis::cmd("SET")
                .arg(&[
                    &key,
                    &value,
                    "XX",
                    "EX",
                    format!("{}", ttl.as_secs()).as_ref(),
                ])
                .clone(),
            Command::Delete { key } => redis::cmd("DEL").arg(&[&key]).clone(),
        }
    }
}
