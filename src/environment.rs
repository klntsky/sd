use crate::command::*;
use crate::shell::*;
use crate::expanded::*;
use async_trait::async_trait;

#[async_trait]
pub trait Env {
    fn initialize() -> Self;
    fn clean_stdin(&mut self) -> ();
    fn print(&mut self, value: String) -> ();
    fn declare(&mut self, var : String, value : String) -> &mut Self;
    fn expand_string(&self, ss : ShellString) -> String;
    fn expand_command(&mut self, tokens : Vec<ShellToken>) -> Vec<ExpandedToken> {
        tokens.iter().map(|token| {
            match token {
                Token::StringToken(shell_string) => {
                    Token::StringToken(
                        self.expand_string(shell_string.clone())
                    )
                }
                Token::Pipe => Token::Pipe,
                Token::Assign => Token::Assign
            }
        }).collect()
    }
    async fn interpret(&mut self, command : Command) -> ();
}
