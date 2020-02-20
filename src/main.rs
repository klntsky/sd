use klntsky_1::shell::*;
use klntsky_1::command::*;

use tokio::prelude::*;
use futures::executor::block_on;
extern crate rustyline;
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
                        println!("{:?}", tokens);
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
    println!("{:?}", shell_token_parser().parse("||||"));

    block_on(run_main());
}
