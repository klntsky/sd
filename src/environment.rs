use crate::command::*;
use crate::shell::*;
use async_trait::async_trait;

#[async_trait]
pub trait Env {
    fn initialize() -> Self;
    fn declare(&mut self, var : String, value : String) -> &mut Self;
    fn expand_string(&self, ss : ShellString) -> String;
    async fn interpret(&mut self, command : Command) -> &mut Self;
}
