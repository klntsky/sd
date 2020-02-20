use crate::environment::*;
use crate::command::*;
use crate::shell::*;
use crate::command::Command::*;
use std::collections::HashMap;

use tokio::fs::File;
use tokio::io::AsyncReadExt;
use async_trait::async_trait;

/// Runtime that represents real-world
pub struct Runtime {
    pub env : HashMap<String, String>,
    pub stdin : Vec<u8>
}

#[async_trait]
impl Env for Runtime {
    fn initialize () -> Runtime {
        Runtime {
            env: HashMap::new(),
            stdin: vec![]
        }
    }

    fn declare(&mut self, var : String, value : String) -> &mut Self {
        self.env.insert(var, value);
        self
    }

    fn expand_string (&self, ss : ShellString) -> String {
        match ss {
            ShellString::WithoutInterpolation(literal) => {
                literal
            }

            ShellString::WithInterpolation(vec) => {
                let mut tmp = String::new();

                for component in vec.iter() {
                    match component {
                        StringComponent::StringLiteral(literal) => {
                            tmp.push_str(literal);
                        }

                        StringComponent::VariableName(variable) => {
                            match self.env.get(variable) {
                                Some(value) => {
                                    tmp.push_str(value);
                                }
                                None => ()
                            }
                        }
                    }
                }

                tmp
            }
        }
    }

    async fn interpret(&mut self, command : Command) -> &mut Self {

        match command {

            CAT(filename) => {
                let file = File::open(filename).await;

                match file {
                    Err(_) => {
                        self.stdin = "No such file".as_bytes().to_vec();
                    }

                    Ok(mut file) => {
                        let mut str = String::new();

                        match file.read_to_string(&mut str).await {
                            Err(_) => {
                                self.stdin = "Can't read file".as_bytes().to_vec();
                            }

                            Ok(_) => {
                                self.stdin = str.as_bytes().to_vec();
                            }
                        }
                    }
                }
            }

            ECHO(string) => {
                self.stdin = string.as_bytes().to_vec();
            }

            WC(string) => {
                self.stdin = "WC".as_bytes().to_vec();
            }

            PWD => {
                self.stdin = "PWD".as_bytes().to_vec();
            }
        };

        self
    }

}

#[cfg(test)]
mod test {

    use super::*;
    use crate::shell::ShellString::*;
    use crate::shell::StringComponent::*;

    #[test]
    fn test_runtime() {
        let mut runtime = Runtime::initialize();

        runtime.declare(
            "foo".to_string(),
            "bar".to_string()
        );

        let tmp = runtime.expand_string(WithInterpolation(vec![
            StringLiteral("bar".to_string()),
            VariableName("foo".to_string()), // expands to "bar"
            StringLiteral("bruh".to_string()),
            VariableName("blah".to_string()) // undefined, expands to ""
        ]));

        assert_eq!(tmp, "barbarbruh");
    }
}
