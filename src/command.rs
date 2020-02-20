use crate::environment::Env;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Debug)]
pub enum Command {
    CAT (String),
    ECHO (String),
    WC (String),
    PWD
    // EXIT,
    // SET (String, String),
}

/// Runtime that represents real-world
pub struct Runtime {
    env : HashMap<String, String>,
    stdin : Vec<u8>
}

pub struct RuntimeMock {
    env : HashMap<String, String>,
    stdin : Vec<u8>,
    files : HashMap<String, String>
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

use Command::*;

pub async fn run<'a> (
    cs : Vec<Command>,
    r : &'a mut Runtime
) {
    for c in cs.iter() {
        match c {
            CAT(filename) => {
                let file = File::open(filename).await;

                match file {
                    Err(_) => {
                        r.stdin = "No such file".as_bytes().to_vec();
                    }

                    Ok(mut file) => {
                        let mut str = String::new();

                        match file.read_to_string(&mut str).await {
                            Err(_) => {
                                r.stdin = "Can't read file".as_bytes().to_vec();
                            }

                            Ok(_) => {
                                r.stdin = str.as_bytes().to_vec();
                            }
                        }
                    }
                }
            }

            ECHO(string) => {
                r.stdin = string.as_bytes().to_vec();
            }

            WC(string) => {
                r.stdin = "WC".as_bytes().to_vec();
            }

            PWD => {
                r.stdin = "PWD".as_bytes().to_vec();
            }
        }
    }
}
