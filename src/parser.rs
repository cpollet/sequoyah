use crate::parser::string::{quoted_string, unquoted_string};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric0, multispace0, multispace1};
use nom::combinator::{map, opt, recognize};
use nom::error::Error;
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded};
use nom::{Finish, IResult, Parser};

pub fn parse_component(input: &str) -> Result<Component<'_>, Error<&str>> {
    component(input).finish().map(|(_, c)| c)
}

pub fn parse_message(input: &str) -> Result<Message<'_>, Error<&str>> {
    message(input).finish().map(|(_, m)| m)
}

mod string {
    // initially from: https://github.com/rust-bakery/nom/blob/main/examples/string.rs
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
    fn text_no_newline<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
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

fn component(input: &str) -> IResult<&str, Component<'_>> {
    (
        tag("component"),
        multispace1,
        alt((
            map((quoted_string, multispace0,tag("as"),multispace1,identifier), |(name,_, _,_,alias)| (name, alias)),
            map(identifier, |ident| (ident.to_string(), ident))
        )),
        multispace0,
        tag(";"),
    )
        .parse(input)
        .map(|(rest, (_tag, _, (name, alias), _, _))| {
            (
                rest,
                Component {
                    kind: ComponentKind::Component,
                    name,
                    alias,
                },
            )
        })
}

/// Parse an attribute value: either an identifier or a quoted string.
fn value(input: &str) -> IResult<&str, Value<'_>> {
    alt((
        map(identifier, Value::Identifier),
        map(quoted_string, Value::Quoted),
    ))
    .parse(input)
}

/// Parse a single `key = value` attribute pair, optionally followed by a `,`.
///
/// Whitespace is tolerated around the `=` and around the trailing comma.
fn attribute(input: &str) -> IResult<&str, Attribute<'_>> {
    delimited(
        multispace0,
        (
            identifier,
            multispace0,
            tag("="),
            multispace0,
            value,
            multispace0,
            opt(tag(",")),
        ),
        multispace0,
    )
    .parse(input)
    .map(|(rest, (key, _, _, _, value, _, _))| (rest, Attribute { key, value }))
}

/// Parse a `[ ... ]` attribute block: comma-separated `key = value` pairs with an
/// optional trailing comma.
fn attributes(input: &str) -> IResult<&str, Vec<Attribute<'_>>> {
    delimited(tag("["), many0(attribute), tag("]")).parse(input)
}

/// Parse a message of the form `a -> b` optionally followed by `[ ... ]` attribute block
/// optionally followed by `: text`.
///
/// The only mandatory part is `a -> b`, where `a` and `b` are identifiers. The text may contain
/// any character except a newline; a newline must be written as the `\n` escape.
fn message(input: &str) -> IResult<&str, Message<'_>> {
    (
        identifier,
        multispace0,
        tag("->"),
        multispace0,
        identifier,
        opt(preceded(multispace0, attributes)),
        opt(preceded(
            multispace0,
            preceded(tag(":"), preceded(multispace0, unquoted_string)),
        )),
    )
        .parse(input)
        .map(|(rest, (from, _, _, _, to, attributes, message))| {
            (
                rest,
                Message {
                    from,
                    to,
                    attributes: attributes.unwrap_or_default(),
                    message,
                },
            )
        })
}

#[allow(unused)]
#[derive(Debug)]
pub struct Component<'input> {
    kind: ComponentKind,
    name: String,
    alias: &'input str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ComponentKind {
    Component,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Message<'input> {
    from: &'input str,
    to: &'input str,
    attributes: Vec<Attribute<'input>>,
    message: Option<String>,
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub struct Attribute<'input> {
    key: &'input str,
    value: Value<'input>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value<'input> {
    Identifier(&'input str),
    Quoted(String),
}

#[cfg(test)]
mod tests {
    use super::{
        Attribute, ComponentKind, Value, component, message, parse_component, parse_message,
    };

    fn assert_component(input: &str, name: &str, alias: &str) {
        let (rest, c) = component(input).expect("expected a component to parse");
        assert_eq!(rest, "", "unexpected trailing input");
        assert_eq!(c.kind, ComponentKind::Component);
        assert_eq!(c.name, name);
        assert_eq!(c.alias, alias);
    }

    // --- `component` ----------------------------------------------------------

    #[test]
    fn quoted_name_with_alias() {
        assert_component("component \"browser\" as b;", "browser", "b");
    }

    #[test]
    fn identifier_name_uses_same_alias() {
        assert_component("component browser;", "browser", "browser");
    }

    #[test]
    fn identifier_name_with_mixed_case() {
        assert_component("component MyComponent;", "MyComponent", "MyComponent");
    }

    #[test]
    fn quoted_name_with_internal_spaces() {
        assert_component(r#"component "my component" as myComp;"#, "my component", "myComp");
    }

    #[test]
    fn empty_quoted_name() {
        assert_component(r#"component "" as x;"#, "", "x");
    }

    #[test]
    fn quoted_name_with_escaped_chars() {
        assert_component(r#"component "a\tb\n" as x;"#, "a\tb\n", "x");
    }

    #[test]
    fn no_space_between_quoted_name_and_as() {
        assert_component(r#"component "browser"as b;"#, "browser", "b");
    }

    #[test]
    fn spaces_after_keyword_are_allowed() {
        assert_component("component    \"browser\" as b;", "browser", "b");
    }

    #[test]
    fn spaces_after_as_are_allowed() {
        assert_component("component \"browser\" as    b;", "browser", "b");
    }

    #[test]
    fn trailing_whitespace_before_semicolon() {
        assert_component("component \"browser\" as b   ;", "browser", "b");
    }

    #[test]
    fn trailing_content_is_returned() {
        let (rest, c) = component("component \"browser\" as b; rest").unwrap();
        assert_eq!(rest, " rest");
        assert_eq!(c.name, "browser");
        assert_eq!(c.alias, "b");
    }

    #[test]
    fn parse_component_wrapper() {
        let c = parse_component("component \"browser\" as b;").unwrap();
        assert_eq!(c.name, "browser");
        assert_eq!(c.alias, "b");
        assert_eq!(c.kind, ComponentKind::Component);
    }

    #[test]
    fn space_after_keyword_is_required() {
        assert!(component("componentbrowser;").is_err());
    }

    #[test]
    fn missing_semicolon() {
        assert!(component("component \"browser\" as b").is_err());
    }

    #[test]
    fn missing_name() {
        assert!(component("component ;").is_err());
    }

    #[test]
    fn missing_component_keyword() {
        assert!(component(r#""browser" as b;"#).is_err());
    }

    #[test]
    fn spaces_before_as_are_allowed() {
        // Whitespace between the closing quote and `as` is tolerated.
        assert_component(r#"component "browser"  as b;"#, "browser", "b");
    }

    #[test]
    fn quoted_name_requires_alias() {
        assert!(component(r#"component "browser";"#).is_err());
    }

    #[test]
    fn missing_alias_after_as() {
        assert!(component(r#"component "browser" as;"#).is_err());
    }

    // --- `message` ------------------------------------------------------------

    #[test]
    fn mandatory_arrow_only() {
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
        let msg = parse_message("b -> s : first line\\nsecond line").unwrap();
        assert_eq!(msg.message, Some("first line\nsecond line".to_string()));
    }

    #[test]
    fn literal_newline_terminates_text() {
        let (rest, msg) = message("b -> s : hello\nworld").unwrap();
        assert_eq!(msg.message, Some("hello".to_string()));
        assert_eq!(rest, "\nworld");
    }

    #[test]
    fn no_attributes() {
        let msg = parse_message("b -> s : GET /path").unwrap();
        assert_eq!(msg.attributes, vec![]);
    }

    #[test]
    fn attributes_with_trailing_comma() {
        let (rest, msg) = message("b -> s [color=red, ] : GET /path").unwrap();
        assert_eq!(rest, "");
        assert_eq!(msg.from, "b");
        assert_eq!(msg.to, "s");
        assert_eq!(
            msg.attributes,
            vec![Attribute {
                key: "color",
                value: Value::Identifier("red"),
            }],
        );
        assert_eq!(msg.message, Some("GET /path".to_string()));
    }

    #[test]
    fn multiple_attributes() {
        let msg = parse_message("a -> b [color=red, size=large,]").unwrap();
        assert_eq!(
            msg.attributes,
            vec![
                Attribute {
                    key: "color",
                    value: Value::Identifier("red"),
                },
                Attribute {
                    key: "size",
                    value: Value::Identifier("large"),
                },
            ],
        );
    }

    #[test]
    fn attribute_quoted_value() {
        let msg = parse_message(r#"a -> b [name="foo bar"]"#).unwrap();
        assert_eq!(
            msg.attributes,
            vec![Attribute {
                key: "name",
                value: Value::Quoted("foo bar".to_string()),
            }],
        );
    }

    #[test]
    fn empty_attribute_block() {
        let msg = parse_message("a -> b []").unwrap();
        assert_eq!(msg.attributes, vec![]);
    }

    #[test]
    fn attributes_without_text() {
        let msg = parse_message("a -> b [color=red]").unwrap();
        assert_eq!(
            msg.attributes,
            vec![Attribute {
                key: "color",
                value: Value::Identifier("red"),
            }],
        );
        assert_eq!(msg.message, None);
    }

    #[test]
    fn missing_arrow_is_an_error() {
        assert!(parse_message("b s").is_err());
    }
}
