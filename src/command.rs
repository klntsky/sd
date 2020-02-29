use crate::shell::*;

use combine::*;
use combine::stream::{Stream};
use std::vec::*;


#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    CAT (Vec<String>),
    ECHO (Vec<String>),
    WC (Vec<String>),
    PWD,
    EXIT,
    SET (String, String),
    EXTERNAL (Vec<String>)
}

parser! {
    /// Match any string (we don't care about quoting style)
    fn string_token[I]()(I) -> String
    where [I: Stream<Token = ExpandedShellToken>]
    {
        satisfy_map(|token| {
            match token {
                ExpandedShellToken::StringToken(t) => Some(t),
                _ => None
            }
        })
    }
}

parser! {
    /// Match a pre-defined constant string (we don't care about quoting style)
    fn literally[I](str : String)(I) -> ()
    where [I: Stream<Token = ExpandedShellToken>]
    {
        satisfy(|token| {
            match token {
                ExpandedShellToken::StringToken(res) =>
                    res.eq(str),
                _ => false
            }
        }).map(|_| ())
    }
}

parser! {
    /// Match a single command (which is anything between pipe symbols)
    fn command_parser[I]()(I) -> Command
    where [I: Stream<Token = ExpandedShellToken>]
    {
        choice((
            literally("cat".to_string()).with(
                many1(string_token()).map(Command::CAT)
            ),
            literally("echo".to_string()).with(
                many(string_token()).map(Command::ECHO)
            ),
            literally("wc".to_string()).with(
                many(string_token()).map(Command::WC)
            ),
            literally("pwd".to_string()).map(|_| Command::PWD),
            literally("exit".to_string()).map(|_| Command::EXIT),
            attempt((
                string_token(),
                token(Token::Assign),
                string_token()
            ).map(|(var, _, value)| {
                Command::SET(var, value)
            })),
            many(string_token()).map(Command::EXTERNAL)
        ))
    }
}

parser! {
    /// Top-level commands parser.
    pub fn commands_parser[I]()(I) -> Vec<Command>
    where [I: Stream<Token = ExpandedShellToken>]
    {
        sep_by1(command_parser(), token(Token::Pipe)).skip(eof())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_command_parser() {
        fn check_parse(
            input: Vec<ExpandedShellToken>,
            expected: Command,
            expected_remaining: Vec<ExpandedShellToken>
        ) {
            let (val, remaining) = command_parser().easy_parse(&input[..]).unwrap();
            assert_eq!(remaining, &expected_remaining[..]);
            assert_eq!(val, expected);
        }

        use Token::*;

        check_parse(vec![
            StringToken("cat".to_string()),
            StringToken("foo".to_string()),
            StringToken("bar".to_string()),
        ], Command::CAT(
            vec!["foo".to_string(), "bar".to_string()]
        ), vec![]);

        check_parse(vec![
            StringToken("time".to_string()),
            StringToken("ls".to_string())
        ], Command::EXTERNAL(
            vec!["time".to_string(), "ls".to_string()]
        ), vec![]);
    }
}
