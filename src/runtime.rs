use crate::environment::*;
use crate::command::*;
use std::process::Stdio;
use crate::command::Command::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process;
use async_trait::async_trait;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::env::{current_dir};

/// Runtime that represents real-world
pub struct Runtime<'a> {
    pub env : HashMap<String, String>,
    pub stdin : Vec<u8>,
    pub editor : &'a mut Editor::<()>
}

#[async_trait]
impl Env for Runtime<'_> {

    fn getline (&mut self) -> Result<String, ReadlineError> {
        self.editor.readline("$ ").map(|line| {
            self.editor.add_history_entry(line.clone().as_str());
            line
        })
    }

    fn clear_stdin(&mut self) {
        self.stdin = vec![];
    }

    fn print(&mut self, value : String) {
        if value.len() > 0 {
            print!("{}", value);
        }
    }

    fn declare(&mut self, var : String, value : String) -> () {
        self.env.insert(var, value);
    }

    fn lookup_variable(&self, variable : String) -> Option<String> {
        self.env.get(&variable).map(|x| x.clone())
    }


    async fn interpret_command(&mut self, command : Command) -> () {
        match command {

            CAT(filenames) => {

                for filename in filenames.iter() {
                    let file = File::open(filename);

                    match file {
                        Err(_) => {
                            self.stdin = (
                                "No such file: ".to_string() + filename
                            ).as_bytes().to_vec();
                        }

                        Ok(mut file) => {
                            let mut str = String::new();

                            match file.read_to_string(&mut str) {
                                Err(_) => {
                                    self.stdin.extend("Can't read file".as_bytes().to_vec());
                                }

                                Ok(_) => {
                                    self.stdin.extend(str.as_bytes().to_vec());
                                }
                            }
                        }
                    }
                }
            }

            ECHO(strings) => {
                self.stdin = strings.iter().fold(
                    "".to_string(),
                    |accum, s| accum + &s + " "
                ).as_bytes().to_vec();
                self.stdin.pop();
            }

            WC (filenames) => {

                // Count lines, words and bytes, using ASCII byte codes
                fn get_stats(contents : &Vec<u8>) -> (u64, u64, u64) {
                    let mut lines : u64 = 1;
                    let mut words : u64 = if contents.len() > 0 { 1 } else { 0 };
                    let mut word = true;
                    let mut bytes : u64 = 0;

                    for byte in contents.iter() {
                        println!("{}", byte.to_string());
                        if *byte == 10 {
                            lines += 1;
                        }

                        if *byte == 32 && word {
                            words += 1;
                            word = false;
                        } else {
                            word = true;
                        }

                        bytes += 1;
                    }

                    (lines, words, bytes)
                }

                // Pretty-print stats, include file names if needed
                let pprint_stats = |(lines, words, bytes) : (u64, u64, u64), filename : String| -> String {

                    let mut output = String::new();
                    output.push_str("\t");
                    output.push_str(&lines.to_string());
                    output.push_str("\t");
                    output.push_str(&words.to_string());
                    output.push_str("\t");
                    output.push_str(&bytes.to_string());

                    if filenames.len() > 1 {
                        output.push_str("\t");
                        output.push_str(&filename);
                    }

                    output.push_str("\n");

                    output
                };


                if filenames.len() == 0 {
                    let stats = pprint_stats(
                        get_stats(&self.stdin), "".to_string()
                    );

                    self.stdin = stats.as_bytes().to_vec();

                    return;
                }

                self.clear_stdin();

                for filename in filenames.iter() {
                    let file = File::open(filename);

                    match file {
                        Err(_) => {
                            self.stdin.extend((
                                "wc: No such file: ".to_string() +
                                    filename +
                                    &"\n".to_string()
                            ).as_bytes().to_vec());
                        }

                        Ok(mut file) => {
                            let mut contents = vec![];

                            match file.read_to_end(&mut contents) {
                                Err(_) => {
                                    self.stdin.extend(
                                        "Can't read file".as_bytes().to_vec()
                                    );
                                }

                                Ok(_) => {
                                    let stats = get_stats(&contents);
                                    self.stdin.extend(
                                        pprint_stats(
                                            stats, filename.to_string()
                                        ).as_bytes().to_vec()
                                    );
                                }
                            }
                        }
                    }
                }
            }

            PWD => {
                self.clear_stdin();

                current_dir().ok()
                    .and_then(|dir| dir.to_str().map(|dir_str| {
                        self.stdin = (*dir_str).as_bytes().to_vec();
                    }));
            }

            EXIT => {
                process::exit(0);
            }

            SET (variable, value) => {
                self.env.insert(variable, value);
            }

            EXTERNAL (commands) => {
                if let Some((binary_name, args)) = commands.split_first() {
                    let result = std::process::Command::new(
                        binary_name
                    ).args(args).stderr(Stdio::piped()).output();

                    match result {
                        Ok(output) => {
                            self.stdin = output.stdout;
                            self.stdin.extend(output.stderr);
                        },

                        _ => {
                            self.stdin = (
                                "No such command: ".to_string() +
                                    binary_name
                            ).as_bytes().to_vec();
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::shell::ShellString::*;
    use crate::shell::StringComponent::*;

    #[test]
    fn test_runtime() {
        let mut runtime = Runtime {
            env: HashMap::new(),
            stdin: vec![],
            editor: &mut Editor::<()>::new()
        };

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
