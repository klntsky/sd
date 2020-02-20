use combine::parser::char::{char};
use combine::{between, choice, many1, parser, sep_by};
use combine::stream::{Stream};
use std::vec::*;
use combine::parser::char::{spaces, string};
use combine::{
    eof, many, none_of, one_of, attempt
};

/// String component is either a `$variable` or just text.
#[derive(Debug, PartialEq, Clone)]
pub enum StringComponent {
    StringLiteral(String),
    VariableName(String)
}

use StringComponent::*;

parser! {
    /// Shell `$variable` parser
    pub fn whitespace[I]()(I) -> ()
    where [I: Stream<Token = char>]
    {
        many1(char(' ')).map(|_ : Vec<char>| ())
    }
}

parser! {
    /// Shell `$variable` parser
    pub fn variable_component[I]()(I) -> StringComponent
    where [I: Stream<Token = char>]
    {
        let allowed_chars =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_";

        char('$').with(
            many1(one_of(
                allowed_chars.chars())).map(
                |chrs : Vec<char>| StringComponent::VariableName(
                    chrs.into_iter().collect()
                )
            )
        )
    }
}

parser! {
    /// Parser for string parts that are not `$variable`s.
    pub fn string_literal_component[I]()(I) -> StringComponent
    where [I: Stream<Token = char>]
    {
        many1(
            none_of("\\\"$".chars()).or(
                escape_sequence()
            )
        ).map(|chrs : Vec<char>|
              StringComponent::StringLiteral(chrs.into_iter().collect())
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ShellString {
    WithInterpolation (Vec<StringComponent>),
    WithoutInterpolation (String)
}
use ShellString::*;

parser! {
    /// Parser for string parts that are not `$variable`s.
    pub fn shell_string[I]()(I) -> ShellString
    where [I: Stream<Token = char>]
    {
        choice((
            quoted_string(),
            fully_quoted_string(),
            unquoted_string()
        ))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ShellToken {
    StringToken(ShellString),
    Pipe,
    Assign
}

use ShellToken::*;

parser! {
    pub fn special_char[I]()(I) -> char
    where [I: Stream<Token = char>]
    {
        choice((
            char('\\'),
            char('"'),
            char('$'),
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
    pub fn quoted_string[I]()(I) -> ShellString
    where [I: Stream<Token = char>]
    {
        between(
            char('"'),
            char('"'),
            many(
                choice((
                    string_literal_component(),
                    variable_component()
                ))
            )
        ).map(WithInterpolation)
    }
}

parser! {
    pub fn fully_quoted_string[I]()(I) -> ShellString
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
        ).map(WithoutInterpolation)
    }
}

parser! {
    pub fn string_unquoted_literal_component[I]()(I) -> StringComponent
    where [I: Stream<Token = char>]
    {
        many1(
            one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_-.,:".chars())
        ).map(
            |chrs : Vec<char>| {
                StringLiteral(chrs.into_iter().collect())
            }
        )
    }
}

parser! {
    pub fn unquoted_string[I]()(I) -> ShellString
    where [I: Stream<Token = char>]
    {
        many1(
            choice((
                attempt(string_unquoted_literal_component()),
                attempt(variable_component())
            ))
        ).map(WithInterpolation)
    }
}

parser! {
    pub fn shell_token_parser_impl[I]()(I) -> Vec<ShellToken>
    where [I: Stream<Token = char>]
    {
        sep_by(
            choice((
                char('|').map(|_| Pipe),
                char('=').map(|_| Assign),
                shell_string().map(StringToken)
            )),
            spaces()
        )
    }
}

parser! {
    pub fn shell_token_parser[I]()(I) -> Vec<ShellToken>
    where [I: Stream<Token = char>]
    {
        shell_token_parser_impl().skip(spaces()).skip(eof())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use combine::parser::Parser;

    #[test]
    fn test_shell_string_parser() {
        fn check_parse(input: &str, expected: ShellString, expected_remaining: &str) {
            let (val, remaining) = shell_string().parse(input).unwrap();
            assert_eq!(remaining, expected_remaining);
            assert_eq!(val, expected);
        }

        check_parse("''", WithoutInterpolation("".to_string()), "");
        check_parse("\"\"", WithInterpolation(vec![]), "");
        check_parse("\"asd bsd\"", WithInterpolation(vec![
            StringLiteral("asd bsd".to_string())
        ]), "");
        check_parse("\"\\\"\"", WithInterpolation(vec![
            StringLiteral("\"".to_string())
        ]), "");
        check_parse("\"\\$foo\"", WithInterpolation(vec![
            StringLiteral("$foo".to_string())
        ]), "");
        check_parse("\"$foo$bar\"", WithInterpolation(vec![
            VariableName("foo".to_string()),
            VariableName("bar".to_string())
        ]), "");
        check_parse("\"$foo $bar\"", WithInterpolation(vec![
            VariableName("foo".to_string()),
            StringLiteral(" ".to_string()),
            VariableName("bar".to_string())
        ]), "");
    }

    #[test]
    fn test_token_parser() {
        fn check_parse(input: &str, expected: Vec<ShellToken>, expected_remaining: &str) {
            let (val, remaining) = shell_token_parser_impl().parse(input).unwrap();
            assert_eq!(remaining, expected_remaining);
            assert_eq!(val, expected);
        }

        // Note: no spaces at the end allowed here
        // `shell_token_parser` is responsible for them

        check_parse("|", vec![Pipe], "");
        check_parse("| |", vec![Pipe, Pipe], "");
        check_parse("FILE=example.txt", vec![
            StringToken(
                WithInterpolation(vec![
                    StringLiteral(
                        "FILE".to_string()
                    )
                ])
            ),
            Assign,
            StringToken(
                WithInterpolation(vec![
                    StringLiteral("example.txt".to_string())
                ])
            )
        ], "");

        check_parse("echo echo", vec![
            StringToken(
                WithInterpolation(vec![
                    StringLiteral("echo".to_string())
                ])
            ),
            StringToken(
                WithInterpolation(vec![
                    StringLiteral("echo".to_string())
                ])
            )
        ], "");

        check_parse("echo \"asd\" | wc", vec![
            StringToken(
                WithInterpolation(vec![
                    StringLiteral("echo".to_string())
                ])
            ),
            StringToken(
                WithInterpolation(vec![
                    StringLiteral("asd".to_string())
                ])
            ),
            Pipe,
            StringToken(
                WithInterpolation(vec![
                    StringLiteral("wc".to_string())
                ])
            ),
        ], "");
   }
}
