use klntsky_1::shell::*;
use klntsky_1::command::*;
use klntsky_1::runtime::*;
use klntsky_1::environment::*;

use tokio::prelude::*;
use futures::executor::block_on;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use combine::{Parser, EasyParser};
use combine::stream::position;

#[cfg(feature = "std")]
use combine::{
    stream::{easy, position::SourcePosition},
    EasyParser,
};

async fn run_main () -> io::Result<()> {

    let mut rl = Editor::<()>::new();

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut rt = Runtime::initialize();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.clone().as_str());

                let result = shell_token_parser()
                    .easy_parse(position::Stream::new(&*line))
                    .map_err(|err| err.map_range(|s| s.to_string()));


                match result {
                    Ok((tokens, _)) => {
                        // println!("{:?}", tokens.clone());

                        let expanded = rt.expand_command(tokens);
                        let result = commands_parser().easy_parse(&expanded[..]);

                        match result {
                            Ok((commands, _)) => {

                                for command in commands.iter() {
                                    rt.interpret(command.clone()).await;
                                }

                                // println!("{:?}", commands.clone());
                                let output = String::from_utf8(rt.stdin.clone());

                                match output {
                                    Ok(str) =>
                                        rt.print(str),
                                    Err(_) =>
                                        rt.print(
                                            "Command output can't be decoded as utf-8.".to_string()
                                        )
                                }

                                rt.clean_stdin();
                            }

                            Err(err) => {
                                println!("No parse!");
                                println!("{:?}", err);
                            }
                        }
                    }

                    Err(err) => {
                        println!("No parse!");
                        println!("{:?}", err);
                    }
                }
            },

            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },

            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },

            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    Ok(())
}

fn main() {
    block_on(run_main());
}
