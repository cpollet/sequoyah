use crate::parser::string::{quoted_string, unquoted_string};
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric0, multispace0, multispace1};
use nom::combinator::{opt, recognize};
use nom::error::Error;
use nom::sequence::{pair, preceded};
use nom::{Finish, IResult, Parser};

pub fn parse_component(input: &str) -> Result<Component, Error<&str>> {
    component(input).finish().map(|(_, c)| c)
}

pub fn parse_message(input: &str) -> Result<Message, Error<&str>> {
    message(input).finish().map(|(_, m)| m)
}

mod string {
    // from: https://github.com/rust-bakery/nom/blob/main/examples/string.rs
    use nom::branch::alt;
    use nom::bytes::complete::is_not;
    use nom::character::complete::{char, multispace1};
    use nom::combinator::{map, value, verify};
    use nom::error::{FromExternalError, ParseError};
    use nom::multi::fold;
    use nom::sequence::{delimited, preceded};
    use nom::{IResult, Parser};

    /// Parse a quoted string.
    pub(super) fn quoted_string<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
    where
        E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        delimited(
            char('"'),
            |input| build_string(input, quoted_str_fragment),
            char('"'),
        )
            .parse(input)
    }

    /// Parse an unquoted string that stops at a newline (or end of input).
    pub(super) fn unquoted_string<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
    where
        E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        build_string(input, unquoted_str_fragment)
    }

    /// Parse a non-empty block of text that doesn't include \ or "
    fn text_no_backslash_no_newline<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, &'a str, E> {
        let not_quote_slash = is_not("\"\\");

        verify(not_quote_slash, |s: &str| !s.is_empty()).parse(input)
    }

    /// Parse a non-empty block of text that doesn't include \ or a newline
    fn text_no_newline<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, &'a str, E> {
        let not_quote_slash_newline = is_not("\\\n");

        verify(not_quote_slash_newline, |s: &str| !s.is_empty()).parse(input)
    }

    /// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
    fn escaped_char<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
    where
        E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        preceded(
            char('\\'),
            alt((
                // todo: parse_unicode,
                value('\n', char('n')),
                value('\r', char('r')),
                value('\t', char('t')),
                value('\u{08}', char('b')),
                value('\u{0C}', char('f')),
                value('\\', char('\\')),
                value('/', char('/')),
                value('"', char('"')),
            )),
        )
        .parse(input)
    }

    /// Parse a backslash, followed by any amount of whitespace. This is used later
    /// to discard any escaped whitespace.
    fn escaped_whitespace<'a, E: ParseError<&'a str>>(
        input: &'a str,
    ) -> IResult<&'a str, &'a str, E> {
        preceded(char('\\'), multispace1).parse(input)
    }

    /// A string fragment contains a fragment of a string being parsed: either
    /// a non-empty Literal (a series of non-escaped characters), a single
    /// parsed escaped character, or a block of escaped whitespace.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum StringFragment<'a> {
        Literal(&'a str),
        EscapedChar(char),
        EscapedWS,
    }

    /// Combine `parse_text_no_backslash_no_newline`, `parse_escaped_char`, and
    /// `parse_escaped_whitespace` into a `StringFragment`.
    fn quoted_str_fragment<'a, E>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
    where
        E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        alt((
            map(text_no_backslash_no_newline, StringFragment::Literal),
            map(escaped_char, StringFragment::EscapedChar),
            value(StringFragment::EscapedWS, escaped_whitespace),
        ))
        .parse(input)
    }

    /// Like `parse_fragment`, but use `parse_text_no_newline` instead of
    /// `parse_text_no_backslash_no_newline`.
    fn unquoted_str_fragment<'a, E>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
    where
        E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        alt((
            map(text_no_newline, StringFragment::Literal),
            map(escaped_char, StringFragment::EscapedChar),
            value(StringFragment::EscapedWS, escaped_whitespace),
        ))
        .parse(input)
    }

    /// Fold a sequence of fragments (produced by `fragment`) into a String.
    fn build_string<'a, E, F>(input: &'a str, fragment: F) -> IResult<&'a str, String, E>
    where
        F: Fn(&'a str) -> IResult<&'a str, StringFragment<'a>, E>,
        E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
    {
        fold(0.., fragment, String::new, |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        })
        .parse(input)
    }
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, alphanumeric0)).parse(input)
}

fn component(input: &str) -> IResult<&str, Component> {
    (
        tag("component"),
        multispace1,
        quoted_string,
        multispace0,
        tag("as"),
        multispace1,
        identifier,
        multispace0,
        tag(";"),
    )
        .parse(input)
        .map(|(rest, (_, _, name, _, _, _, alias, _, _))| {
            (
                rest,
                Component {
                    kind: ComponentKind::Component,
                    name: name.to_string(),
                    alias: Some(alias.to_string()),
                },
            )
        })
}

/// Parse a message of the form `a -> b` optionally followed by `: text`.
///
/// The only mandatory part is `a -> b`, where `a` and `b` are identifiers. The
/// `:` and the text that follows are optional. The text may contain any
/// character except a newline; a newline must be written as the `\n` escape.
fn message(input: &str) -> IResult<&str, Message> {
    (
        identifier,
        multispace0,
        tag("->"),
        multispace0,
        identifier,
        opt(preceded(
            multispace0,
            preceded(tag(":"), preceded(multispace0, unquoted_string)),
        )),
    )
        .parse(input)
        .map(|(rest, (from, _, _, _, to, message))| {
            (
                rest,
                Message {
                    from: from.to_string(),
                    to: to.to_string(),
                    message,
                },
            )
        })
}

#[derive(Debug)]
pub struct Component {
    kind: ComponentKind,
    name: String,
    alias: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum ComponentKind {
    Component,
}

#[derive(Debug)]
pub struct Message {
    from: String,
    to: String,
    message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{message, parse_message};

    #[test]
    fn mandatory_arrow_only() {
        // The only mandatory part is `a -> b`.
        let (rest, msg) = message("s -> d").unwrap();
        assert_eq!(rest, "");
        assert_eq!(msg.from, "s");
        assert_eq!(msg.to, "d");
        assert_eq!(msg.message, None);
    }

    #[test]
    fn with_text() {
        let (rest, msg) = message("b -> s : GET /path").unwrap();
        assert_eq!(rest, "");
        assert_eq!(msg.from, "b");
        assert_eq!(msg.to, "s");
        assert_eq!(msg.message, Some("GET /path".to_string()));
    }

    #[test]
    fn no_space_around_arrow() {
        let msg = parse_message("b->s").unwrap();
        assert_eq!(msg.from, "b");
        assert_eq!(msg.to, "s");
        assert_eq!(msg.message, None);
    }

    #[test]
    fn extra_whitespace_is_allowed() {
        // Whitespace around the arrow and before the text is tolerated; the text
        // itself keeps any trailing whitespace (it is "any character but newline").
        let (rest, msg) = message("a   ->   b   :   hello   ").unwrap();
        assert_eq!(rest, "");
        assert_eq!(msg.from, "a");
        assert_eq!(msg.to, "b");
        assert_eq!(msg.message, Some("hello   ".to_string()));
    }

    #[test]
    fn empty_text_after_colon() {
        let msg = parse_message("b -> s :").unwrap();
        assert_eq!(msg.message, Some(String::new()));
    }

    #[test]
    fn newline_must_be_escaped() {
        // A `\n` escape becomes an actual newline in the parsed text.
        let msg = parse_message("b -> s : first line\\nsecond line").unwrap();
        assert_eq!(msg.message, Some("first line\nsecond line".to_string()));
    }

    #[test]
    fn literal_newline_terminates_text() {
        // A literal newline stops the text and is left in the remaining input.
        let (rest, msg) = message("b -> s : hello\nworld").unwrap();
        assert_eq!(msg.message, Some("hello".to_string()));
        assert_eq!(rest, "\nworld");
    }

    #[test]
    fn brackets_are_ignored() {
        // Bracket metadata is not part of this grammar, so `b -> s` parses and the
        // remainder is left untouched.
        let (rest, msg) = message("b -> s [color=red, ] : GET /path").unwrap();
        assert_eq!(msg.from, "b");
        assert_eq!(msg.to, "s");
        assert_eq!(msg.message, None);
        assert_eq!(rest, " [color=red, ] : GET /path");
    }

    #[test]
    fn missing_arrow_is_an_error() {
        assert!(parse_message("b s").is_err());
    }
}
