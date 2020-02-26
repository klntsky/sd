use crate::command::*;
use crate::shell::*;
use async_trait::async_trait;
use rustyline::error::ReadlineError;

#[async_trait]
pub trait Env {
    fn clear_stdin(&mut self) -> ();
    fn print(&mut self, value: String) -> ();
    fn declare(&mut self, var : String, value : String);
    fn lookup_variable(&self, variable : String) -> Option<String>;
    fn getline (&mut self) -> Result<String, ReadlineError>;
    fn expand_string (
        &self,
        ss : ShellString
    ) -> String {

        use ShellString::*;

        match ss {
            WithoutInterpolation(literal) => {
                literal
            }

            WithInterpolation(vec) => {
                let mut tmp = String::new();

                for component in vec.iter() {
                    use StringComponent::*;

                    match component {
                        StringLiteral(literal) => {
                            tmp.push_str(literal);
                        }

                        VariableName(variable) => {
                            self.lookup_variable(
                                variable.to_string()
                            ).map(|value| tmp.push_str(&value));
                        }
                    }
                }

                tmp
            }
        }
    }

    fn expand_command (
        &mut self,
        tokens : Vec<ShellToken>
    ) -> Vec<ExpandedShellToken> {
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

    async fn interpret (
        &mut self,
        commands : Vec<Command>
    ) -> () {
        for command in commands.iter() {
            self.interpret_command(command.clone()).await;
        }
    }

    async fn interpret_command(&mut self, command : Command);
}
