use crate::environment::*;
use crate::command::*;
use crate::command::Command::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process;
use async_trait::async_trait;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::env::{current_dir};
use std::io::{BufRead,BufReader};
use duct::cmd;
use dia_args;
use regex::RegexBuilder;

/// Runtime that represents real-world
pub struct Runtime<'a> {
    pub env : HashMap<String, String>,
    pub stdin : Vec<u8>,
    pub editor : &'a mut Editor::<()>,
    /// This flag is used to implement streaming from external commands.
    /// We set it to false when dealing with external commands, and to
    /// true otherwise.
    /// If the output comes from some external command, we stream it
    /// line-by-line directly to stdout, and then our "shell" should not print it
    /// twice when program exits. This flag is used to prevent output duplication.
    pub should_print : bool
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

    async fn interpret_command(&mut self, command : Command, is_last : bool) -> () {
        self.should_print = true;

        match command {

            CAT(filenames) => {
                self.clear_stdin();

                for filename in filenames.iter() {
                    let file = File::open(filename);

                    match file {
                        Err(_) => {
                            self.stdin.extend((
                                "No such file: ".to_string() + filename + "\n"
                            ).as_bytes().to_vec());
                        }

                        Ok(mut file) => {
                            let mut str = String::new();

                            match file.read_to_string(&mut str) {
                                Err(_) => {
                                    self.stdin.extend("Can't read file\n".as_bytes().to_vec());
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
                        if *byte == 10 {
                            word = false;
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
                self.clear_stdin();
                self.env.insert(variable, value);
            }

            EXTERNAL (commands) => {
                self.should_print = false;

                if let Some((binary_name, args)) = commands.split_first() {

                    // Construct a new environment for child process
                    let mut env_map: HashMap<String, String> =
                        std::env::vars().collect();

                    for (key, value) in self.env.iter() {
                        env_map.insert(key.into(), value.into());
                    }

                    // construct a reader handle
                    let mb_reader_handle = cmd(binary_name, args)
                        .full_env(&env_map)
                        .stdin_bytes(self.stdin.clone())
                        .stderr_to_stdout()
                        .reader();

                    self.clear_stdin();

                    match mb_reader_handle {
                        Ok(reader_handle) => {
                            let lines = BufReader::new(reader_handle).lines();

                            // Read everything line-by-line, printing directly
                            // to stdout if needed.
                            for mb_line in lines {
                                if let Ok(line) = mb_line {

                                    self.stdin.extend(line.as_bytes().to_vec());
                                    self.stdin.extend("\n".as_bytes().to_vec());

                                    if is_last {
                                        println!("{}", line);
                                    }

                                } else {
                                    break;
                                }
                            }
                        }

                        _ => {
                            self.should_print = true;
                            self.stdin = (
                                "No such command: ".to_string() +
                                    binary_name
                            ).as_bytes().to_vec();
                        }
                    }
                } else {
                    self.clear_stdin();
                }
            }
            GREP (raw_args) => {
                const ARG_CASE_INSENSITIVE: &[&str] = &["-i"];
                const ARG_WHOLE_WORDS: &[&str] = &["-w"];

                if let Ok(mut args) = dia_args::parse_strings(raw_args.iter()) {
                    let case_insensitive = match args.get(ARG_CASE_INSENSITIVE) {
                        Ok(Some(true)) => true,
                        _ => false
                    };

                    let whole_words = match args.get(ARG_WHOLE_WORDS) {
                        Ok(Some(true)) => true,
                        _ => false
                    };

                    let lines = match args.get::<u32>(&["-A"]) {
                        Ok(Some(n)) => n,
                        _ => 1
                    };

                    let other_args = args.take_args();

                    let stdin = String::from_utf8(self.stdin.clone()).ok();
                    self.clear_stdin();

                    let mut process_regexps = |needle : String, haystack : String| {
                        let regexp = RegexBuilder::new(&needle)
                            .case_insensitive(case_insensitive)
                            .multi_line(true)
                            .build();

                        if let Ok(rx) = regexp {
                            let mut remaining = 0;
                            let mut line_start = 0;
                            let mut chars_iter = haystack.char_indices();
                            let mut mat_iter = rx.find_iter(haystack.as_str());
                            let mut linevec = vec![];

                            let mut tmp = String::new();
                            let mut last_ix = 0;

                            while let Some((ix, chr)) = chars_iter.next() {
                                last_ix = ix;

                                if chr == '\n' {
                                    tmp.push(chr);
                                    linevec.push((line_start, ix, tmp.clone()));
                                    tmp = String::new();
                                    line_start = ix + 1;
                                } else {
                                    tmp.push(chr);
                                }
                            }

                            if line_start + 1 != last_ix {
                                linevec.push((line_start, last_ix, tmp.clone()));
                            }

                            // Check if there is a word between start and end indices.
                            let is_word = |start, end| {
                                (start == 0 || !haystack.chars().nth(start-1).unwrap().is_alphanumeric()) &&
                                    (end == haystack.len() - 1 || !haystack.chars().nth(end).unwrap().is_alphanumeric())
                            };

                            if let Some(mut next_mat) = mat_iter.next() {
                                for (start, end, line) in linevec.iter() {
                                    while next_mat.start() < *end {
                                        if next_mat.start() >= *start && next_mat.start() < *end {
                                            if !whole_words || is_word(next_mat.start(), next_mat.end()) {
                                                remaining = lines;
                                            }
                                        }

                                        match mat_iter.next() {
                                            Some(next_next_mat) => {
                                                next_mat = next_next_mat;
                                            },
                                            _ => break
                                        }
                                    }

                                    if next_mat.start() >= *start && next_mat.start() < *end {
                                        if !whole_words || is_word(next_mat.start(), next_mat.end()) {
                                            remaining = lines;
                                        }
                                    }

                                    if remaining > 0 {
                                        self.stdin.extend(line.as_bytes().to_vec());
                                        remaining -= 1;
                                    }
                                }
                            } else {
                                // no matches
                            }

                        } else {
                            self.stdin = "grep: incorrect regexp! see https://docs.rs/regex/1.3.4/regex/#syntax".as_bytes().to_vec();
                        }
                    };

                    match other_args {
                        // from stdin
                        Some(vec) if vec.len() == 1 => {
                            let needle = (&vec[0]).to_string();
                            if let Some(haystack) = stdin {
                                process_regexps(needle, haystack);
                            } else {
                                self.stdin = "grep: stdin does not contain valid utf-8".as_bytes().to_vec();
                            }
                        }

                        // from file
                        Some(vec) if vec.len() == 2 => {
                            let needle = &vec[0];
                            let filename = &vec[1];

                            if let Ok(mut file) = File::open(filename) {
                                let mut haystack = String::new();
                                match file.read_to_string(&mut haystack) {
                                    Err(_) => {
                                        self.stdin.extend("grep: Can't read file\n".as_bytes().to_vec());
                                    }

                                    Ok(_) => {
                                        process_regexps(needle.to_string(), haystack);
                                    }
                                }
                            } else {
                                self.stdin.extend((
                                    "grep: No such file: ".to_string() + filename + "\n"
                                ).as_bytes().to_vec());
                            }

                        }

                        _ => {
                            self.stdin = (
                                "grep: incorrect arguments!"
                            ).as_bytes().to_vec();
                        }
                    }

                } else {
                    self.stdin = (
                        "grep: Incorrect arguments!"
                    ).as_bytes().to_vec();
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
    use futures::executor::block_on;

    #[test]
    fn test_runtime_string_interpolation() {
        let mut runtime = Runtime {
            env: HashMap::new(),
            stdin: vec![],
            editor: &mut Editor::<()>::new(),
            should_print: true
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

    #[test]
    fn test_runtime_echo_wc() {

        let mut runtime = Runtime {
            env: HashMap::new(),
            stdin: vec![],
            editor: &mut Editor::<()>::new(),
            should_print: true
        };

        block_on(runtime.interpret_command(
            Command::ECHO(vec![
                "a".to_string(),
                "a".to_string()
            ]),
            false
        ));

        assert_eq!(runtime.stdin.len(), 3);

        block_on(runtime.interpret_command(
            Command::WC(vec![]),
            false
        ));

        assert_eq!(
            String::from_utf8(runtime.stdin.clone()).unwrap(),
            "\t1\t2\t3\n".to_string()
        );

        block_on(runtime.interpret_command(
            Command::ECHO(vec![
                "a\n b".to_string(),
            ]),
            false
        ));

        block_on(runtime.interpret_command(
            Command::WC(vec![]), false
        ));

        assert_eq!(
            String::from_utf8(runtime.stdin.clone()).unwrap(),
            "\t2\t2\t4\n".to_string()
        );
    }

    #[test]
    fn test_args () {
        const ARG_CASE_INSENSITIVE: &[&str] = &["-i"];

        if let Ok(args) = dia_args::parse_strings(["foo", "-i"].iter()) {
            match args.get(ARG_CASE_INSENSITIVE) {
                Ok(Some(true)) => {}
                _ => panic!("fail")
            }
        } else {
            panic!("Fail");
        }

        if let Ok(args) = dia_args::parse_strings(["foo"].iter()) {
            match args.get::<bool>(ARG_CASE_INSENSITIVE) {
                Ok(None) => {}
                _ => panic!("fail")
            }
        } else {
            panic!("Fail");
        }

    }
}
