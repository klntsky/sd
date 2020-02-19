extern crate combine;

use combine::parser::char::{char};
use combine::{between, choice, many1, parser, sep_by};
use combine::stream::{Stream};
use std::vec::*;
use combine::parser::char::{spaces, string};
use combine::{
    eof, many, none_of, one_of
};

#[derive(Debug, PartialEq)]
pub enum ShellToken {
    Quoted(String),
    FullyQuoted(String),
    Unquoted(String),
    Pipe
}

use ShellToken::*;

parser! {
    pub fn special_char[I]()(I) -> char
    where [I: Stream<Token = char>]
    {
        choice((
            char('\\'),
            char('"')
        ))
    }
}

parser! {
    pub fn escape_sequence[I]()(I) -> char
    where [I: Stream<Token = char>]
    {
        (
            char('\\'),
            special_char()
        ).map(|(_, c)| c)
    }
}

parser! {
    pub fn fully_quoted_escape_sequence[I]()(I) -> char
    where [I: Stream<Token = char>]
    {
        (
            char('\\'),
            char('\'')
        ).map(|(_, _)| '\'')
    }
}

parser! {
    pub fn quoted_string_parser[I]()(I) -> String
    where [I: Stream<Token = char>]
    {
        between(
            char('"'),
            char('"'),
            many(
                none_of("\\\"".chars()).or(
                    escape_sequence()
                )
            )
        ).map(|chrs : Vec<char>|
              chrs.into_iter().collect()
        )
    }
}

parser! {
    pub fn fully_quoted_string_parser[I]()(I) -> String
    where [I: Stream<Token = char>]
    {
        between(
            char('\''),
            char('\''),
            many(
                none_of("\\'".chars()).or(
                    fully_quoted_escape_sequence()
                )
            )
        ).map(|chrs : Vec<char>|
              chrs.into_iter().collect()
        )
    }
}

parser! {
    pub fn unquoted_string_parser[I]()(I) -> String
    where [I: Stream<Token = char>]
    {
        many1(one_of("abcdefghijklmnopqrstuvwxyz1234567890_-".chars())).map(
            |chrs : Vec<char>| chrs.into_iter().collect()
        )
    }
}

parser! {
    pub fn shell_token_parser_impl[I]()(I) -> Vec<ShellToken>
    where [I: Stream<Token = char>]
    {
        sep_by(
            choice((
                quoted_string_parser().map(
                    Quoted
                ),
                unquoted_string_parser().map(
                    Unquoted
                ),
                fully_quoted_string_parser().map(
                    FullyQuoted
                ),
                string("|").map(|_| Pipe)
            )),
            spaces()
        )
    }
}

parser! {
    pub fn shell_token_parser[I]()(I) -> Vec<ShellToken>
    where [I: Stream<Token = char>]
    {
        shell_token_parser_impl().skip(eof())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use combine::parser::Parser;

    #[test]
    fn test_token_parser() {
        fn check_parse(input: &str, expected: Vec<ShellToken>, expected_remaining: &str) {
            let (val, remaining) = shell_token_parser_impl().parse(input).unwrap();
            assert_eq!(remaining, expected_remaining);
            assert_eq!(val, expected);
        }

        check_parse("|", vec![Pipe], "");
        check_parse("||", vec![Pipe, Pipe], "");

        check_parse("echo echo", vec![
            Unquoted("echo".to_string()),
            Unquoted("echo".to_string())
        ], "");

        check_parse("echo \"asd\" | wc", vec![
            Unquoted("echo".to_string()),
            Quoted("asd".to_string()),
            Pipe,
            Unquoted("wc".to_string())
        ], "");

        check_parse("\"asd\"", vec![Quoted("asd".to_string())], "");
        check_parse("\"a\\\"sd\"", vec![Quoted("a\"sd".to_string())], "");
        check_parse("\"a\\\\sd\"", vec![Quoted("a\\sd".to_string())], "");
        check_parse("\"a\\\\sd\\\\\"", vec![Quoted("a\\sd\\".to_string())], "");
        check_parse("'asd'", vec![FullyQuoted("asd".to_string())], "");
        check_parse("'a\\'sd'", vec![FullyQuoted("a'sd".to_string())], "");
        check_parse(".foo", vec![], ".foo");
    }
}
