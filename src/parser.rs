use crate::parser::string::{quoted_string, unquoted_string};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric0, multispace1, not_line_ending};
use nom::combinator::{all_consuming, value as val};
use nom::combinator::{eof, map, opt, recognize};
use nom::error::{Error, ParseError};
use nom::multi::{many0, many0_count};
use nom::sequence::{delimited, pair, preceded};
use nom::{Finish, IResult, Parser};

pub fn parse(input: &str) -> Result<Graph<'_>, Error<&str>> {
    all_consuming((graph, whitespace0))
        .parse(input)
        .finish()
        .map(|(_, (g, _))| g)
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

pub fn skip<I, E, F>(parser: F) -> impl Parser<I, Output = (), Error = E>
where
    E: ParseError<I>,
    F: Parser<I, Error = E>,
{
    val((), parser)
}

/// Matches a comment, i.e `//` followed by non line ending characters. Does not consume the line
/// ending character.
fn comment<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str>,
{
    skip(pair(tag("//"), not_line_ending)).parse(input)
}

/// Consume any amount of comments and whitespace characters.
fn whitespace0<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str>,
{
    skip(many0_count(alt((skip(multispace1), comment)))).parse(input)
}

fn ws<'a, O, E, F>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
where
    E: ParseError<&'a str>,
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(whitespace0, inner, whitespace0)
}

fn sym<'a, E>(s: &'a str) -> impl Parser<&'a str, Output = &'a str, Error = E>
where
    E: ParseError<&'a str>,
{
    ws(tag(s))
}

/// Consume any amount of comments and whitespace characters, but at least one whitespace character
/// (or eof).
fn whitespace1(input: &str) -> IResult<&str, ()> {
    skip((
        // skip a comment, if any
        opt(comment),
        // we either want a whitespace or an eof
        skip(alt((multispace1, eof))),
        // we consume any amount of whitespaces / comments that may remain
        whitespace0,
    ))
    .parse(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, alphanumeric0)).parse(input)
}

fn component(input: &str) -> IResult<&str, Component<'_>> {
    (
        alt((
            val(ComponentKind::Component, tag("participant")),
            val(ComponentKind::Component, tag("component")),
            val(ComponentKind::Actor, tag("actor")),
            val(ComponentKind::Boundary, tag("boundary")),
            val(ComponentKind::Control, tag("control")),
            val(ComponentKind::Entity, tag("entity")),
            val(ComponentKind::Database, tag("database")),
            val(ComponentKind::Collections, tag("collections")),
            val(ComponentKind::Queue, tag("queue")),
        )),
        alt((
            map(
                (ws(quoted_string), tag("as"), whitespace1, identifier),
                |(name, _, _, alias)| (name, alias),
            ),
            map((whitespace1, identifier), |(_, ident)| {
                (ident.to_string(), ident)
            }),
        )),
        sym(";"),
    )
        .parse(input)
        .map(|(rest, (kind, (name, alias), ..))| (rest, Component { kind, name, alias }))
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
    (ws(identifier), sym("="), ws(value), opt(sym(",")))
        .parse(input)
        .map(|(rest, (key, _, value, _))| (rest, Attribute { key, value }))
}

/// Parse a `[ ... ]` attribute block: comma-separated `key = value` pairs with an
/// optional trailing comma.
fn attributes(input: &str) -> IResult<&str, Vec<Attribute<'_>>> {
    delimited(sym("["), many0(attribute), sym("]")).parse(input)
}

fn components(input: &str) -> IResult<&str, Vec<Component<'_>>> {
    many0(ws(component)).parse(input)
}

/// Parse a message of the form `a -> b` optionally followed by `[ ... ]` attribute block
/// optionally followed by `: text`.
///
/// The only mandatory part is `a -> b`, where `a` and `b` are identifiers. The text may contain
/// any character except a newline; a newline must be written as the `\n` escape.
fn message(input: &str) -> IResult<&str, Message<'_>> {
    (
        ws(identifier),
        arrow,
        ws(identifier),
        opt(attributes).map(Option::unwrap_or_default),
        opt(preceded(sym(":"), unquoted_string)),
        opt(activation),
    )
        .parse(input)
        .map(
            |(rest, (from, arrow, to, attributes, message, activation))| {
                (
                    rest,
                    Message {
                        from,
                        arrow,
                        to,
                        attributes,
                        message,
                        activation,
                    },
                )
            },
        )
}

fn arrow(input: &str) -> IResult<&str, Arrow> {
    fn left(input: &str) -> IResult<&str, ArrowHead> {
        opt(alt((
            val(ArrowHead::Thin, tag("<<")),
            val(ArrowHead::Thick, tag("<")),
            val(ArrowHead::Lost, tag("x")),
        )))
        .parse(input)
        .map(|(rest, head)| (rest, head.unwrap_or_default()))
    }
    fn right(input: &str) -> IResult<&str, ArrowHead> {
        opt(alt((
            val(ArrowHead::Thin, tag(">>")),
            val(ArrowHead::Thick, tag(">")),
            val(ArrowHead::Lost, tag("x")),
        )))
        .parse(input)
        .map(|(rest, head)| (rest, head.unwrap_or_default()))
    }
    fn body(input: &str) -> IResult<&str, ArrowBody> {
        alt((
            val(ArrowBody::Dash, tag("--")),
            val(ArrowBody::Solid, tag("-")),
        ))
        .parse(input)
    }
    fn dot(input: &str) -> IResult<&str, bool> {
        opt(alt((tag("o"), tag("*"))))
            .parse(input)
            .map(|(rest, body)| (rest, body.is_some()))
    }

    (dot, left, body, right, dot)
        .parse(input)
        .map(|(rest, (ld, l, b, r, rd))| {
            (
                rest,
                Arrow {
                    left_dot: ld,
                    left_head: l,
                    body: b,
                    right_head: r,
                    right_dot: rd,
                },
            )
        })
}

fn activation(input: &str) -> IResult<&str, Activation<'_>> {
    (sym("{"), messages, opt(ret_val), sym("}"))
        .parse(input)
        .map(|(rest, (_, messages, ret_val, _))| {
            (
                rest,
                Activation {
                    messages,
                    ret_val: ret_val.map(|rv| rv.1),
                },
            )
        })
}

fn ret_val(input: &str) -> IResult<&str, (Vec<Attribute<'_>>, String)> {
    preceded(
        delimited(whitespace0, tag("return"), whitespace1),
        (opt(attributes), unquoted_string),
    )
    .parse(input)
    .map(|(rest, (attributes, ret_val))| (rest, (attributes.unwrap_or_default(), ret_val)))
}

fn messages(input: &str) -> IResult<&str, Vec<Message<'_>>> {
    many0(message).parse(input)
}

fn graph(input: &str) -> IResult<&str, Graph<'_>> {
    (components, messages)
        .parse(input)
        .map(|(rest, (components, messages))| {
            (rest, {
                Graph {
                    components,
                    messages,
                }
            })
        })
}

#[allow(unused)]
#[derive(Debug)]
pub struct Graph<'input> {
    pub components: Vec<Component<'input>>,
    pub messages: Vec<Message<'input>>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Component<'input> {
    pub kind: ComponentKind,
    pub name: String,
    pub alias: &'input str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    Component,
    Actor,
    Boundary,
    Control,
    Entity,
    Database,
    Collections,
    Queue,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Message<'input> {
    pub from: &'input str,
    pub arrow: Arrow,
    pub to: &'input str,
    pub attributes: Vec<Attribute<'input>>,
    pub message: Option<String>,
    pub activation: Option<Activation<'input>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Arrow {
    pub left_dot: bool,
    pub left_head: ArrowHead,
    pub body: ArrowBody,
    pub right_head: ArrowHead,
    pub right_dot: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum ArrowHead {
    #[default]
    None,
    Thick,
    Thin,
    Lost,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArrowBody {
    Solid,
    Dash,
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

#[allow(unused)]
#[derive(Debug)]
pub struct Activation<'input> {
    pub messages: Vec<Message<'input>>,
    pub ret_val: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        Arrow, ArrowBody, ArrowHead, Attribute, ComponentKind, Value, comment, component,
        components, message, whitespace0, whitespace1,
    };
    use insta::assert_debug_snapshot;
    use nom::error::Error;

    // --- `comment` ------------------------------------------------------------

    #[test]
    fn comment_single_line_no_newline() {
        let (rest, _) = comment::<Error<&str>>("//comment").unwrap();
        assert_eq!(rest, "");
    }

    #[test]
    fn comment_single_line_newline() {
        let (rest, _) = comment::<Error<&str>>("//comment\n").unwrap();
        assert_eq!(rest, "\n");
    }

    #[test]
    fn comment_single_line_newline_and_rest() {
        let (rest, _) = comment::<Error<&str>>("//comment\nrest").unwrap();
        assert_eq!(rest, "\nrest");
    }

    // --- `whitespace0` --------------------------------------------------------

    #[test]
    fn whitespace0_only_whitespaces() {
        let (rest, _) = whitespace0::<Error<&str>>(" \n\trest").unwrap();
        assert_eq!(rest, "rest");
    }

    #[test]
    fn whitespace0_whitespaces_then_comment() {
        let (rest, _) = whitespace0::<Error<&str>>("\n\t // comment \nrest").unwrap();
        assert_eq!(rest, "rest");
    }

    #[test]
    fn whitespace0_comment() {
        let (rest, _) = whitespace0::<Error<&str>>("// comment\nrest").unwrap();
        assert_eq!(rest, "rest");
    }

    #[test]
    fn whitespace0_comment_no_rest() {
        let (rest, _) = whitespace0::<Error<&str>>("// comment").unwrap();
        assert_eq!(rest, "");
    }

    #[test]
    fn whitespace0_non_whitespaces() {
        let (rest, _) = whitespace0::<Error<&str>>("rest").unwrap();
        assert_eq!(rest, "rest");
    }

    // --- `whitespace1` --------------------------------------------------------

    #[test]
    fn whitespace1_only_whitespaces() {
        let (rest, _) = whitespace1(" \n\trest").unwrap();
        assert_eq!(rest, "rest");
    }

    #[test]
    fn whitespace1_whitespaces_then_comment() {
        let (rest, _) = whitespace1("\n\t // comment \nrest").unwrap();
        assert_eq!(rest, "rest");
    }

    #[test]
    fn whitespace1_comment() {
        let (rest, _) = whitespace1("// comment\nrest").unwrap();
        assert_eq!(rest, "rest");
    }

    #[test]
    fn whitespace1_comment_no_rest() {
        let (rest, _) = whitespace1("// comment").unwrap();
        assert_eq!(rest, "");
    }

    #[test]
    fn whitespace1_non_whitespaces() {
        assert!(whitespace1("rest").is_err());
    }

    // --- `component` ----------------------------------------------------------

    macro_rules! assert_component {
        ($input:expr, $name:expr, $alias:expr) => {
            assert_component!($input, $name, $alias, "");
        };
        ($input:expr, $name:expr, $alias:expr, $rest:expr) => {
            let (rest, c) = component($input).expect("expected a component to parse");
            assert_eq!(rest, $rest);
            assert_eq!(c.kind, ComponentKind::Component);
            assert_eq!(c.name, $name);
            assert_eq!(c.alias, $alias);
        };
    }

    #[test]
    fn component_quoted_name_with_alias() {
        assert_component!("component \"browser\" as b;", "browser", "b");
    }

    #[test]
    fn component_identifier_name_uses_same_alias() {
        assert_component!("component browser;", "browser", "browser");
    }

    #[test]
    fn component_empty_quoted_name() {
        assert_component!(r#"component "" as x;"#, "", "x");
    }

    #[test]
    fn component_quoted_name_with_escaped_chars() {
        assert_component!(r#"component "a\tb\n" as x;"#, "a\tb\n", "x");
    }

    #[test]
    fn component_no_space_between_component_and_quoted_name() {
        assert_component!(r#"component"browser" as b;"#, "browser", "b");
    }

    #[test]
    fn component_no_space_between_quoted_name_and_as() {
        assert_component!(r#"component "browser"as b;"#, "browser", "b");
    }

    #[test]
    fn component_trailing_content_is_returned() {
        assert_component!("component \"browser\" as b; rest", "browser", "b", "rest");
    }

    #[test]
    fn component_space_after_keyword_is_required() {
        assert!(component("componentbrowser;").is_err());
    }

    #[test]
    fn component_missing_semicolon() {
        assert!(component("component \"browser\" as b").is_err());
    }

    #[test]
    fn component_missing_name() {
        assert!(component("component ;").is_err());
    }

    #[test]
    fn component_missing_component_keyword() {
        assert!(component(r#""browser" as b;"#).is_err());
    }

    #[test]
    fn component_quoted_name_requires_alias() {
        assert!(component(r#"component "browser";"#).is_err());
    }

    #[test]
    fn component_missing_alias_after_as() {
        assert!(component(r#"component "browser" as;"#).is_err());
    }

    // --- `parse_components` (multiple components) ---------------------------

    #[test]
    fn components_multiple() {
        let (rest, c) = components(r#"component browser; component server;"#).unwrap();
        assert_eq!(rest, "");
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].name, "browser");
        assert_eq!(c[0].alias, "browser");
        assert_eq!(c[1].name, "server");
        assert_eq!(c[1].alias, "server");
    }

    #[test]
    fn components_trailing_content_is_returned() {
        let (rest, c) = components(r#"component browser; rest"#).unwrap();
        assert_eq!(rest, "rest");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].name, "browser");
        assert_eq!(c[0].alias, "browser");
    }

    // --- `message` ------------------------------------------------------------

    macro_rules! assert_message {
        ($input:expr, $from:expr, $arrow:expr, $to:expr) => {
            assert_message!(@int $input, $from, $arrow, $to, None, vec![], "");
        };
        ($input:expr, $from:expr, $arrow:expr, $to:expr, attrs: $attrs:expr) => {
            assert_message!(@int $input, $from, $arrow, $to, None, $attrs, "");
        };
        ($input:expr, $from:expr, $arrow:expr, $to:expr, $message:expr) => {
            assert_message!(@int $input, $from, $arrow, $to, Some($message), vec![], "");
        };
        ($input:expr, $from:expr, $arrow:expr, $to:expr, $message:expr, rest: $rest:expr) => {
            assert_message!(@int $input, $from, $arrow, $to, Some($message), vec![], $rest);
        };
        ($input:expr, $from:expr, $arrow:expr, $to:expr, $message:expr, attrs: $attrs:expr) => {
            assert_message!(@int $input, $from, $arrow, $to, Some($message), $attrs, "");
        };
        (@int $input:expr, $from:expr, $arrow:expr, $to:expr, $message:expr, $attrs:expr, $rest:expr) => {
            let (rest, msg) = message($input).unwrap();
            assert_eq!(rest, $rest);
            assert_eq!(msg.from, $from);
            assert_eq!(msg.arrow, $arrow);
            assert_eq!(msg.to, $to);
            assert_eq!(msg.message.as_deref(), $message);
            assert_eq!(msg.attributes, $attrs);
        };
    }

    #[test]
    fn message_mandatory_arrow_only() {
        assert_message!(
            "s -> d",
            "s",
            Arrow {
                left_dot: false,
                left_head: Default::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_dashed_arrow() {
        assert_message!(
            "s --> d",
            "s",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Dash,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_reverse_arrow() {
        assert_message!(
            "s <- d",
            "s",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::Thick,
                body: ArrowBody::Solid,
                right_head: Default::default(),
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_thin_arrow() {
        assert_message!(
            "s ->> d",
            "s",
            Arrow {
                left_dot: false,
                left_head: Default::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thin,
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_thin_reverse_arrow() {
        assert_message!(
            "s <<- d",
            "s",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::Thin,
                body: ArrowBody::Solid,
                right_head: Default::default(),
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_lost_arrow() {
        assert_message!(
            "s -x d",
            "s",
            Arrow {
                left_dot: false,
                left_head: Default::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Lost,
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_lost_reverse_arrow() {
        assert_message!(
            "s x- d",
            "s",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::Lost,
                body: ArrowBody::Solid,
                right_head: Default::default(),
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_dot_arrow() {
        assert_message!(
            "s ->o d",
            "s",
            Arrow {
                left_dot: false,
                left_head: Default::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: true,
            },
            "d"
        );
    }

    #[test]
    fn message_dot_reverse_arrow() {
        assert_message!(
            "s o-> d",
            "s",
            Arrow {
                left_dot: true,
                left_head: Default::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_double_arrow() {
        assert_message!(
            "s <-> d",
            "s",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::Thick,
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "d"
        );
    }

    #[test]
    fn message_with_text() {
        assert_message!(
            "b -> s : GET /path",
            "b",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "s",
            "GET /path"
        );
    }

    #[test]
    fn message_no_space_around_arrow() {
        assert_message!(
            "b->s",
            "b",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "s"
        );
    }

    #[test]
    fn empty_text_after_colon() {
        assert_message!(
            "b -> s : ",
            "b",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "s",
            ""
        );
    }

    #[test]
    fn message_newline_must_be_escaped() {
        assert_message!(
            "b -> s : first line\\nsecond line",
            "b",
            Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            },
            "s",
            "first line\nsecond line"
        );
    }

    #[test]
    fn message_literal_newline_terminates_text() {
        assert_message!("b -> s : hello\nworld", "b", Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            }, "s", "hello", rest: "\nworld");
    }

    #[test]
    fn message_attributes_with_trailing_comma() {
        assert_message!("b -> s [color=red, ] : GET /path", "b", Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            }, "s", "GET /path", attrs: vec![Attribute {
            key: "color",
            value: Value::Identifier("red"),
        }]);
    }

    #[test]
    fn message_multiple_attributes() {
        assert_message!("a -> b [color=red, size=large,]", "a", Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            }, "b", attrs:vec![
            Attribute {
                key: "color",
                value: Value::Identifier("red"),
            },
            Attribute {
                key: "size",
                value: Value::Identifier("large"),
            },
        ]);
    }

    #[test]
    fn message_attribute_quoted_value() {
        assert_message!(r#"a -> b [name="foo bar"]"#, "a", Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            }, "b", attrs: vec![Attribute {
            key: "name",
            value: Value::Quoted("foo bar".to_string()),
        }]);
    }

    #[test]
    fn message_empty_attribute_block() {
        assert_message!("a -> b []", "a", Arrow {
                left_dot: false,
                left_head: ArrowHead::default(),
                body: ArrowBody::Solid,
                right_head: ArrowHead::Thick,
                right_dot: false,
            }, "b", attrs: vec![]);
    }

    #[test]
    fn message_missing_arrow_is_an_error() {
        assert!(message("b s").is_err());
    }

    #[test]
    fn message_with_activation() {
        assert_debug_snapshot!(
            message(
                r#"
a -> b
{
    b -> c
    {
        return 1
    }
    return 2
}
"#
            )
            .unwrap()
        );
    }
}
