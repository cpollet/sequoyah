mod arrows;

use crate::ast::Arrow;
use regex::Regex;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::mem;
use std::str::Chars;
use std::sync::LazyLock;

pub struct Lexer<'input> {
    input: &'input str,
    remaining: &'input str,
    start: usize,
    line: usize,
    column: usize,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            remaining: input,
            start: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn peekable(self) -> PeekableLexer<'input> {
        PeekableLexer {
            lexer: self,
            peeked: Option::default(),
        }
    }

    fn make_keyword<F>(&self, mut consumed: usize, remaining: &str, f: F) -> Token<'input>
    where
        F: FnOnce(Span) -> Token<'input>,
    {
        let mut chars = self.remaining[consumed..].chars();
        let mut remaining = remaining.chars();

        loop {
            let (c, r) = (chars.next(), remaining.next());

            #[allow(clippy::match_same_arms)]
            match (c, r) {
                (Some(c), Some(r)) if c == r => {
                    // both chars match, we keep consuming
                    consumed += 1;
                }
                (None, None) => {
                    // both reached the end, we have a keyword
                    return f(Span {
                        start: self.start,
                        length: consumed,
                        line: self.line,
                        column: self.column,
                    });
                }
                (Some(c), None) if !c.is_ascii_alphanumeric() => {
                    // input does not contain an alphanumeric char, and we reached the end of the
                    // keyword, we make it
                    return f(Span {
                        start: self.start,
                        length: consumed,
                        line: self.line,
                        column: self.column,
                    });
                }
                (Some(c), _) if c.is_ascii_alphanumeric() => {
                    // both chars don't match, but input contains an alphanumeric char, we make an
                    // identifier
                    consumed += 1;
                    return self.make_identifier(consumed);
                }
                (Some(_), _) => {
                    // both chars don't match, and input does not contain an alphanumeric char, we
                    // make an identifier
                    return self.make_identifier(consumed);
                }
                (None, Some(_)) => {
                    // we reached the end of input without matching the keyword, we make an
                    // identifier
                    return self.make_identifier(consumed);
                }
            }
        }
    }

    fn make_identifier(&self, consumed: usize) -> Token<'input> {
        let chars = self.remaining[consumed..].chars();
        let consumed = consumed + chars.take_while(char::is_ascii_alphanumeric).count();

        Token::Identifier(
            Span {
                start: self.start,
                length: consumed,
                line: self.line,
                column: self.column,
            },
            &self.remaining[0..consumed],
        )
    }

    fn make_quoted_string(&self, mut chars: Chars) -> Token<'input> {
        //todo use descape crate?
        let mut consumed = 1; // the initial '"' is already consumed

        let mut owned = None;

        loop {
            match chars.next() {
                Some('"') => {
                    consumed += 1;
                    break;
                }
                Some('\\') => {
                    if owned.is_none() {
                        owned = Some(self.remaining[1..consumed].to_string());
                    }
                    consumed += 1;
                    match chars.next() {
                        Some('n') => {
                            consumed += 1;
                            if let Some(owned) = owned.as_mut() {
                                owned.push('\n');
                            }
                        }
                        Some('t') => {
                            consumed += 1;
                            if let Some(owned) = owned.as_mut() {
                                owned.push('\t');
                            }
                        }
                        Some(c) => {
                            consumed += 1;
                            if let Some(owned) = owned.as_mut() {
                                owned.push(c);
                            }
                        }
                        None => (),
                    }
                }
                Some(c) => {
                    consumed += 1;
                    if let Some(owned) = owned.as_mut() {
                        owned.push(c);
                    }
                }
                None => todo!("Report unterminated quoted string"),
            }
        }

        Token::QuotedString(
            Span {
                start: self.start,
                length: consumed,
                line: self.line,
                column: self.column,
            },
            owned.map_or_else(
                || Cow::Borrowed(&self.remaining[1..consumed - 1]),
                Cow::Owned,
            ),
        )
    }

    fn make_arrow(&self) -> Token<'input> {
        let consumed = 1 + self.remaining[1..]
            .chars()
            .take_while(|c| *c == 'o' || *c == 'x' || *c == '<' || *c == '>' || *c == '-')
            .count();

        let span = Span {
            start: self.start,
            length: consumed,
            line: self.line,
            column: self.column,
        };
        let arrow_str = &self.remaining[0..consumed];

        match arrows::ARROWS.get(arrow_str) {
            Some(arrow) => Token::Arrow(span, arrow),
            None => Token::Invalid(span, arrow_str),
        }
    }

    fn make_color(&self) -> Token<'input> {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#[a-fA-F0-9]{6}$").unwrap());

        let code = &self.remaining[0..7];
        let span = Span {
            start: self.start,
            length: 7,
            line: self.line,
            column: self.column,
        };

        if RE.is_match(code) {
            Token::HexColor(span, code)
        } else {
            Token::Invalid(span, code)
        }
    }

    /// Skips whitespaces and comments, returns `(consumed chars, current line, current column)`
    fn skip_whitespace(&mut self) -> bool {
        let mut chars = self.remaining.chars();
        let mut consumed = 0;

        loop {
            match chars.next() {
                Some('/') if let Some('/') = chars.next() => {
                    consumed += 2;
                    self.column += 2;
                    loop {
                        match chars.next() {
                            Some('\n') => {
                                consumed += 1;
                                self.line += 1;
                                self.column = 0;
                                break;
                            }
                            Some(_) => {
                                consumed += 1;
                                self.column += 1;
                            }
                            None => break,
                        }
                    }
                }
                Some('\n') => {
                    self.line += 1;
                    self.column = 0;
                    consumed += 1;
                }
                Some(c) if c.is_ascii_whitespace() => {
                    self.column += 1;
                    consumed += 1;
                }
                _ => break,
            }
        }

        self.remaining = &self.remaining[consumed..];
        self.start += consumed;
        consumed > 0
    }

    pub fn location(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    pub fn unquoted_string(&mut self) -> Token<'input> {
        //todo use descape crate?
        self.skip_whitespace();

        let mut chars = self.remaining.chars();
        let mut consumed = 0;
        let line = self.line;
        let column = self.column;

        let mut owned = None;

        let read_newline = loop {
            match chars.next() {
                None => break false,
                Some('\n') => {
                    consumed += 1;
                    self.column = 0;
                    self.line += 1;
                    break true;
                }
                Some('\\') => {
                    if owned.is_none() {
                        owned = Some(self.remaining[1..consumed].to_string());
                    }
                    match chars.next() {
                        Some('n') => {
                            consumed += 2;
                            self.column += 2;
                            if let Some(owned) = owned.as_mut() {
                                owned.push('\n');
                            }
                        }
                        Some('t') => {
                            consumed += 2;
                            self.column += 2;
                            if let Some(owned) = owned.as_mut() {
                                owned.push('\t');
                            }
                        }
                        Some(c) => {
                            consumed += 2;
                            self.column += 2;
                            if let Some(owned) = owned.as_mut() {
                                owned.push(c);
                            }
                        }
                        None => {
                            consumed += 1;
                            self.column += 1;
                        }
                    }
                }
                Some(c) => {
                    consumed += 1;
                    self.column += 1;
                    if let Some(owned) = owned.as_mut() {
                        owned.push(c);
                    }
                }
            }
        };

        let string_len = if read_newline { consumed - 1 } else { consumed };

        let token = Token::UnquotedString(
            Span {
                start: self.start,
                length: string_len,
                line,
                column,
            },
            owned.map_or_else(|| Cow::Borrowed(&self.remaining[0..string_len]), Cow::Owned),
        );

        self.remaining = &self.remaining[consumed..];
        self.start += consumed;

        token
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token<'input>;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let mut chars = self.remaining.chars();

        let token = match chars.next()? {
            'a' => match chars.next() {
                Some('s') => self.make_keyword(2, "", Token::As),
                Some('c') => self.make_keyword(2, "tor", Token::Actor),
                _ => self.make_identifier(1),
            },
            'b' => self.make_keyword(1, "oundary", Token::Boundary),
            'c' => match chars.next() {
                Some('o') => match chars.next() {
                    Some('l') => self.make_keyword(3, "lections", Token::Collections),
                    Some('m') => self.make_keyword(3, "ponent", Token::Component),
                    Some('n') => self.make_keyword(3, "trol", Token::Control),
                    _ => self.make_identifier(2),
                },
                _ => self.make_identifier(1),
            },
            'd' => self.make_keyword(1, "atabase", Token::Database),
            'e' => self.make_keyword(1, "ntity", Token::Entity),
            'o' => match chars.next() {
                Some('v') => self.make_keyword(2, "erlay", Token::Overlay),
                Some('-' | '<') => self.make_arrow(),
                _ => self.make_identifier(1),
            },
            'q' => self.make_keyword(1, "ueue", Token::Queue),
            'r' => self.make_keyword(1, "eturn", Token::Return),
            'x' => match chars.next() {
                Some('-') => self.make_arrow(),
                _ => self.make_identifier(1),
            },
            c if c.is_ascii_alphabetic() => self.make_identifier(1),
            '-' | '<' => self.make_arrow(),
            ':' => Token::Colon(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            ';' => Token::Semicolon(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            ',' => Token::Comma(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            '"' => self.make_quoted_string(chars),
            '[' => Token::LBracket(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            ']' => Token::RBracket(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            '{' => Token::LCurly(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            '}' => Token::RCurly(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            '=' => Token::Equal(Span {
                start: self.start,
                length: 1,
                line: self.line,
                column: self.column,
            }),
            '#' => self.make_color(),
            _ => Token::Invalid(
                Span {
                    start: self.start,
                    length: 1,
                    line: self.line,
                    column: self.column,
                },
                &self.remaining[0..1],
            ),
        };

        self.remaining = &self.remaining[token.span().length..];
        self.start += token.span().length;
        self.column += token.span().length;

        Some(token)
    }
}

pub struct PeekableLexer<'input> {
    lexer: Lexer<'input>,
    peeked: Option<Token<'input>>,
}

impl<'input> PeekableLexer<'input> {
    pub fn skip_whitespace(&mut self) -> bool {
        debug_assert!(self.peeked.is_none());
        self.lexer.skip_whitespace()
    }

    pub fn unquoted_string(&mut self) -> Token<'input> {
        debug_assert!(self.peeked.is_none());
        self.lexer.unquoted_string()
    }

    pub fn location(&self) -> (usize, usize) {
        self.lexer.location()
    }

    pub fn peek(&mut self) -> Option<&Token<'input>> {
        if self.peeked.is_none() {
            let mut token = self.lexer.next();
            mem::swap(&mut self.peeked, &mut token);
        }
        self.peeked.as_ref()
    }

    pub fn next_if<F>(&mut self, f: F) -> Option<Token<'input>>
    where
        F: Fn(&Token<'input>) -> bool,
    {
        if let Some(peeked) = self.peek()
            && f(peeked)
        {
            self.next()
        } else {
            None
        }
    }
}

impl<'input> Iterator for PeekableLexer<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        mem::take(&mut self.peeked).or_else(|| self.lexer.next())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Token<'input> {
    Component(Span),
    Actor(Span),
    Boundary(Span),
    Control(Span),
    Entity(Span),
    Database(Span),
    Collections(Span),
    Queue(Span),

    Identifier(Span, &'input str),

    QuotedString(Span, Cow<'input, str>),
    UnquotedString(Span, Cow<'input, str>), // todo merge with QuotedString?
    HexColor(Span, &'input str),

    Arrow(Span, &'static Arrow),

    As(Span),
    Return(Span),
    Overlay(Span),

    LBracket(Span),
    RBracket(Span),
    LCurly(Span),
    RCurly(Span),
    Equal(Span),
    Comma(Span),
    Colon(Span),
    Semicolon(Span),

    Invalid(Span, &'input str),
}

impl Token<'_> {
    pub fn lexer(input: &str) -> Lexer<'_> {
        Lexer::new(input)
    }

    pub fn span(&self) -> &Span {
        match self {
            Token::Component(span)
            | Token::Actor(span)
            | Token::Boundary(span)
            | Token::Control(span)
            | Token::Entity(span)
            | Token::Database(span)
            | Token::Collections(span)
            | Token::Queue(span)
            | Token::Identifier(span, _)
            | Token::QuotedString(span, _)
            | Token::UnquotedString(span, _)
            | Token::HexColor(span, _)
            | Token::Arrow(span, _)
            | Token::As(span)
            | Token::Return(span)
            | Token::Overlay(span)
            | Token::LBracket(span)
            | Token::RBracket(span)
            | Token::LCurly(span)
            | Token::RCurly(span)
            | Token::Equal(span)
            | Token::Comma(span)
            | Token::Colon(span)
            | Token::Semicolon(span)
            | Token::Invalid(span, _) => span,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Component(_) => write!(f, "Component"),
            Token::Actor(_) => write!(f, "Actor"),
            Token::Boundary(_) => write!(f, "Boundary"),
            Token::Control(_) => write!(f, "Control"),
            Token::Entity(_) => write!(f, "Entity"),
            Token::Database(_) => write!(f, "Database"),
            Token::Collections(_) => write!(f, "Collections"),
            Token::Queue(_) => write!(f, "Queue"),
            Token::Identifier(_, identifier) => write!(f, "Identifier: {identifier}"),
            Token::QuotedString(_, string) => write!(f, "QuotedString: {string}"),
            Token::UnquotedString(_, string) => write!(f, "UnquotedString: {string}"),
            Token::HexColor(_, color) => write!(f, "Color: {color}"),
            Token::Arrow(_, _) => write!(f, "Arrow"),
            Token::As(_) => write!(f, "As"),
            Token::Return(_) => write!(f, "Return"),
            Token::Overlay(_) => write!(f, "Overlay"),
            Token::LBracket(_) => write!(f, "LBracket"),
            Token::RBracket(_) => write!(f, "RBracket"),
            Token::LCurly(_) => write!(f, "LCurly"),
            Token::RCurly(_) => write!(f, "RCurly"),
            Token::Equal(_) => write!(f, "Equal"),
            Token::Comma(_) => write!(f, "Comma"),
            Token::Colon(_) => write!(f, "Colon"),
            Token::Semicolon(_) => write!(f, "Semicolon"),
            Token::Invalid(_, invalid) => write!(f, "Invalid: {invalid}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Span {
    start: usize,
    length: usize,
    line: usize,
    column: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line + 1, self.column + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_token {
        ($str:expr, $expected:expr, $remaining:expr) => {
            let mut lexer = Lexer::new($str);
            let token = lexer.next();
            assert_eq!(
                token,
                Some($expected),
                "expected {:?} but got {:?}",
                $expected,
                token
            );
            assert_eq!(lexer.remaining, $remaining);
        };
    }

    #[test]
    fn keyword_actor() {
        assert_token!(
            "actor^",
            Token::Actor(Span {
                start: 0,
                length: 5,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_as() {
        assert_token!(
            "as^",
            Token::As(Span {
                start: 0,
                length: 2,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_return() {
        assert_token!(
            "return^",
            Token::Return(Span {
                start: 0,
                length: 6,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_overlay() {
        assert_token!(
            "overlay^",
            Token::Overlay(Span {
                start: 0,
                length: 7,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_collections() {
        assert_token!(
            "collections^",
            Token::Collections(Span {
                start: 0,
                length: 11,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_component() {
        assert_token!(
            "component^",
            Token::Component(Span {
                start: 0,
                length: 9,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_control() {
        assert_token!(
            "control^",
            Token::Control(Span {
                start: 0,
                length: 7,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_database() {
        assert_token!(
            "database^",
            Token::Database(Span {
                start: 0,
                length: 8,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_entity() {
        assert_token!(
            "entity^",
            Token::Entity(Span {
                start: 0,
                length: 6,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn keyword_queue() {
        assert_token!(
            "queue^",
            Token::Queue(Span {
                start: 0,
                length: 5,
                line: 0,
                column: 0
            }),
            "^"
        );
    }

    #[test]
    fn identifier_a() {
        assert_token!(
            "a^",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 1,
                    line: 0,
                    column: 0
                },
                "a"
            ),
            "^"
        );
    }

    #[test]
    fn identifier_other() {
        assert_token!(
            "other^",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 5,
                    line: 0,
                    column: 0
                },
                "other"
            ),
            "^"
        );
    }

    #[test]
    fn identifier_xyz() {
        assert_token!(
            "xyz^",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 3,
                    line: 0,
                    column: 0
                },
                "xyz"
            ),
            "^"
        );
    }

    #[test]
    fn identifier_ad() {
        assert_token!(
            "ad^",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 2,
                    line: 0,
                    column: 0
                },
                "ad"
            ),
            "^"
        );
    }

    #[test]
    fn identifier_asd() {
        assert_token!(
            "asd^",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 3,
                    line: 0,
                    column: 0
                },
                "asd"
            ),
            "^"
        );
    }

    #[test]
    fn identifier_collection() {
        assert_token!(
            "collection^",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 10,
                    line: 0,
                    column: 0
                },
                "collection"
            ),
            "^"
        );
    }

    #[test]
    fn identifier_connection() {
        assert_token!(
            "connection^",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 10,
                    line: 0,
                    column: 0
                },
                "connection"
            ),
            "^"
        );
    }

    #[test]
    fn skip_whitespaces() {
        assert_token!(
            " \t\n ;^",
            Token::Semicolon(Span {
                start: 4,
                length: 1,
                line: 1,
                column: 1
            }),
            "^"
        );
    }

    #[test]
    fn skip_comments() {
        assert_token!(
            "//comment\nas//more comment",
            Token::As(Span {
                start: 10,
                length: 2,
                line: 1,
                column: 0
            }),
            "//more comment"
        );
    }

    #[test]
    fn quoted_string() {
        assert_token!(
            "\"string\"^",
            Token::QuotedString(
                Span {
                    start: 0,
                    length: 8,
                    line: 0,
                    column: 0
                },
                Cow::Borrowed("string")
            ),
            "^"
        );
    }

    #[test]
    fn unquoted_string_newline_terminated() {
        let mut lexer = Lexer::new(" unquoted string\n^");
        let token = lexer.unquoted_string();
        assert_eq!(
            token,
            Token::UnquotedString(
                Span {
                    start: 1,
                    length: 15,
                    line: 0,
                    column: 1
                },
                Cow::Borrowed("unquoted string")
            )
        );
        assert_eq!(lexer.remaining, "^");
    }

    #[test]
    fn unquoted_string_eoi_terminated() {
        let mut lexer = Lexer::new(" unquoted string");
        let token = lexer.unquoted_string();
        assert_eq!(
            token,
            Token::UnquotedString(
                Span {
                    start: 1,
                    length: 15,
                    line: 0,
                    column: 1
                },
                Cow::Borrowed("unquoted string")
            )
        );
        assert_eq!(lexer.remaining, "");
    }

    #[test]
    fn quoted_string_with_escaped_chars() {
        assert_token!(
            "\" \\\" \\n \\t \\x \"^",
            Token::QuotedString(
                Span {
                    start: 0,
                    length: 15,
                    line: 0,
                    column: 0
                },
                Cow::Owned(" \" \n \t x ".to_string())
            ),
            "^"
        );
    }

    #[test]
    fn comma() {
        assert_token!(
            ",^",
            Token::Comma(Span {
                start: 0,
                length: 1,
                line: 0,
                column: 0
            },),
            "^"
        );
    }

    #[test]
    fn lbracket() {
        assert_token!(
            "[^",
            Token::LBracket(Span {
                start: 0,
                length: 1,
                line: 0,
                column: 0
            },),
            "^"
        );
    }

    #[test]
    fn rbracket() {
        assert_token!(
            "]^",
            Token::RBracket(Span {
                start: 0,
                length: 1,
                line: 0,
                column: 0
            },),
            "^"
        );
    }

    #[test]
    fn lcurly() {
        assert_token!(
            "{^",
            Token::LCurly(Span {
                start: 0,
                length: 1,
                line: 0,
                column: 0
            },),
            "^"
        );
    }

    #[test]
    fn rcurly() {
        assert_token!(
            "}^",
            Token::RCurly(Span {
                start: 0,
                length: 1,
                line: 0,
                column: 0
            },),
            "^"
        );
    }

    #[test]
    fn colon() {
        assert_token!(
            ":^",
            Token::Colon(Span {
                start: 0,
                length: 1,
                line: 0,
                column: 0
            },),
            "^"
        );
    }

    #[test]
    fn semicolon() {
        assert_token!(
            ";^",
            Token::Semicolon(Span {
                start: 0,
                length: 1,
                line: 0,
                column: 0
            },),
            "^"
        );
    }

    #[test]
    fn hex_color() {
        assert_token!(
            "#000000^",
            Token::HexColor(
                Span {
                    start: 0,
                    length: 7,
                    line: 0,
                    column: 0
                },
                "#000000"
            ),
            "^"
        );
    }

    #[test]
    fn arrows() {
        for (s, a) in arrows::ARROWS.into_iter() {
            assert_token!(
                s,
                Token::Arrow(
                    Span {
                        start: 0,
                        length: s.len(),
                        line: 0,
                        column: 0
                    },
                    a
                ),
                ""
            );
        }
    }
}
