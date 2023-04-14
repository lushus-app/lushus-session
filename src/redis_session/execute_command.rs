use crate::redis_session::{commands::Command, error::RedisError};

#[async_trait::async_trait(?Send)]
pub trait ExecuteCommand {
    async fn execute_command<T: redis::FromRedisValue>(
        &self,
        command: Command,
    ) -> Result<T, RedisError>;
}
