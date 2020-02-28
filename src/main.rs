use klntsky_1::shell::*;
use klntsky_1::command::*;
use klntsky_1::runtime::*;
use klntsky_1::environment::*;

use std::collections::HashMap;
use futures::executor::block_on;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use combine::{EasyParser};
use combine::stream::position;

#[cfg(feature = "std")]
use combine::{
    stream::{easy, position::SourcePosition},
    EasyParser,
};

async fn run_main (rt : &mut Runtime<'_>) {

    loop {
        match rt.getline() {

            Ok(line) => {
                let result = shell_token_parser()
                    .easy_parse(position::Stream::new(&*line))
                    .map_err(|err| err.map_range(|s| s.to_string()));

                match result {
                    Ok((tokens, _)) => {

                        let expanded = rt.expand_command(tokens);
                        let result = commands_parser().easy_parse(&expanded[..]);

                        match result.ok() {
                            Some((commands, _)) => {

                                rt.interpret(commands).await;

                                if rt.should_print {
                                    match String::from_utf8(rt.stdin.clone()).ok() {
                                        Some(str) => rt.print(str),
                                        None => rt.print(
                                            "Output can't be decoded as utf-8.".to_string()
                                        )
                                    }
                                }

                                rt.clear_stdin();
                            }

                            None => {
                                rt.print("No parse!".to_string());
                            }
                        }
                    }

                    Err(_err) => {
                        rt.print("No parse!".to_string());
                    }
                }
            },

            Err(ReadlineError::Interrupted) => {
                println!("Bye!");
                break
            },

            Err(ReadlineError::Eof) => {
                break
            },

            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    };
}

fn main() {
    let mut runtime = Runtime {
        env: HashMap::new(),
        stdin: vec![],
        editor: &mut Editor::<()>::new(),
        should_print: true
    };

    block_on(run_main(&mut runtime));
}
