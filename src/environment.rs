use crate::command::*;
use async_trait::async_trait;

#[async_trait]
pub trait Env {
    fn initialize() -> Self;
    fn declare(&mut self, var : String, value : String) -> &mut Self;
    async fn interpret(&mut self, command : Command) -> &mut Self;
}
