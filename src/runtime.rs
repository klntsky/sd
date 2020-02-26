use crate::environment::*;
use crate::command::*;
use crate::shell::*;
use crate::command::Command::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process;
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

    fn clean_stdin(&mut self) {
        self.stdin = vec![];
    }

    fn print(&mut self, value : String) {
        if value.len() > 0 {
            println!("{}", value);
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

    async fn interpret(&mut self, command : Command) -> () {
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
                let mut tmp : String = String::new();

                for string in strings.iter() {
                    tmp.push_str(string);
                    tmp.push_str(" ");
                }

                self.stdin = tmp.as_bytes().to_vec();
            }

            WC (filenames) => {

                // Count lines, words and bytes, using ASCII byte codes
                fn get_stats(contents : &Vec<u8>) -> (u64, u64, u64) {
                    let mut words : u64 = 0;
                    let mut lines : u64 = 0;
                    let mut bytes : u64 = 0;

                    for byte in contents.iter() {
                        if *byte == 32 {
                            words += 1;
                        }

                        if *byte == 13 {
                            lines += 1;
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

                self.clean_stdin();

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
                self.stdin = "PWD".as_bytes().to_vec();
            }

            EXIT => {
                process::exit(0);
            }

            SET (variable, value) => {
                self.env.insert(variable, value);
            }

            EXTERNAL (commands) => {

            }
        };
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
