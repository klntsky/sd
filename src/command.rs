use crate::shell::*;

extern crate combine;
use combine::*;
use combine::parser::char::{char};
use combine::{between, choice, many1, parser, sep_by};
use combine::stream::{Stream};
use std::vec::*;
// use combine::parser::char::{spaces};
use combine::{
    eof, many, none_of, one_of, satisfy
};

#[derive(Debug, Clone)]
pub enum Command {
    CAT (String),
    ECHO (String),
    WC (String),
    PWD
    // EXIT,
    // SET (String, String),
}

parser! {
    /// Match any string (we don't care about quoting style)
    pub fn string_token[I]()(I) -> ShellToken
    where [I: Stream<Token = ShellToken>]
    {
        satisfy_map(|token| {
            match token {
                t@ShellToken::StringToken(_)    => Some(t),
                _ => None
            }
        })
    }
}

// parser! {
//     /// Match a pre-defined constant string (we don't care about quoting style)
//     pub fn literally[I](str : String)(I) -> ()
//     where [I: Stream<Token = ShellToken>]
//     {
//         satisfy(|token| {
//             match token {
//                 ShellToken::StringToken(res) =>
//                     res.eq(str),
//                 ShellToken::FullyQuoted(res) =>
//                     res.eq(str),
//                 ShellToken::Quoted(res) =>
//                     res.eq(str),
//                 _ => false
//             }
//         }).map(|_| ())
//     }
// }

// parser! {
//     pub fn special_char[I]()(I) -> Command
//     where [I: Stream<Token = ShellToken>]
//     {
//         choice((
//             literally("cat".to_string()).with(
//                 string_token().map(Command::CAT)
//             ),
//             literally("echo".to_string()).with(
//                 string_token().map(Command::ECHO)
//             ),
//             literally("wc".to_string()).with(
//                 string_token().map(Command::WC)
//             )
//         ))
//     }
// }

//                 satisfy(|x| x == ShellToken::Unquoted("cat".to_string())),

//             satisfy(|x| x == ShellToken::Unquoted("cat".to_string()))
//         ))
//     }
// }


// pub async fn run<'a> (
//     cs : Vec<Command>,
//     r : &'a mut Runtime
// ) {
//     for c in cs.iter() {
//         match c {
//             CAT(filename) => {
//                 let file = File::open(filename).await;

//                 match file {
//                     Err(_) => {
//                         r.stdin = "No such file".as_bytes().to_vec();
//                     }

//                     Ok(mut file) => {
//                         let mut str = String::new();

//                         match file.read_to_string(&mut str).await {
//                             Err(_) => {
//                                 r.stdin = "Can't read file".as_bytes().to_vec();
//                             }

//                             Ok(_) => {
//                                 r.stdin = str.as_bytes().to_vec();
//                             }
//                         }
//                     }
//                 }
//             }

//             ECHO(string) => {
//                 r.stdin = string.as_bytes().to_vec();
//             }

//             WC(string) => {
//                 r.stdin = "WC".as_bytes().to_vec();
//             }

//             PWD => {
//                 r.stdin = "PWD".as_bytes().to_vec();
//             }
//         }
//     }
// }
