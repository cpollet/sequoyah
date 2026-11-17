use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::Chars;

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

    fn make_keyword<F>(
        &self,
        mut chars: Chars,
        mut consumed: usize,
        remaining: &str,
        f: F,
    ) -> (usize, Token<'input>)
    where
        F: FnOnce(Span) -> Token<'input>,
    {
        let mut remaining = remaining.chars();

        loop {
            let (c, r) = (chars.next(), remaining.next());

            match (c, r) {
                (Some(c), Some(r)) if c == r => {
                    // both chars match, we keep consuming
                    consumed += 1;
                    continue;
                }
                (None, None) => {
                    // both reached the end, we have a keyword
                    return (
                        consumed,
                        f(Span {
                            start: self.start,
                            length: consumed,
                            line: self.line,
                            column: self.column,
                        }),
                    );
                }
                (Some(c), None) if !c.is_ascii_alphanumeric() => {
                    // input does not contain an alphanumeric char, and we reached the end of the
                    // keyword, we make it
                    return (
                        consumed,
                        f(Span {
                            start: self.start,
                            length: consumed,
                            line: self.line,
                            column: self.column,
                        }),
                    );
                }
                (Some(c), _) if c.is_ascii_alphanumeric() => {
                    // both chars don't match, but input contains an alphanumeric char, we make an
                    // identifier
                    consumed += 1;
                    return self.make_identifier(chars, consumed);
                }
                (Some(_), _) => {
                    // both chars don't match, and input does not contain an alphanumeric char, we
                    // make an identifier
                    return self.make_identifier(chars, consumed);
                }
                (None, Some(_)) => {
                    // we reached the end of input without matching the keyword, we make an
                    // identifier
                    return self.make_identifier(chars, consumed);
                }
            }
        }
    }

    fn make_identifier(&self, chars: Chars, consumed: usize) -> (usize, Token<'input>) {
        let consumed = consumed + chars.take_while(|c| c.is_ascii_alphanumeric()).count();
        (
            consumed,
            Token::Identifier(
                Span {
                    start: self.start,
                    length: consumed,
                    line: self.line,
                    column: self.column,
                },
                &self.remaining[0..consumed],
            ),
        )
    }

    fn make_quoted_string(&self, mut chars: Chars) -> (usize, Token<'input>) {
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

        (
            consumed,
            Token::QuotedString(
                Span {
                    start: self.start,
                    length: consumed,
                    line: self.line,
                    column: self.column,
                },
                owned
                    .map(Cow::Owned)
                    .unwrap_or_else(|| Cow::Borrowed(&self.remaining[1..consumed - 1])),
            ),
        )
    }

    /// Skips whitespaces and comments, returns `(consumed chars, current line, current column)`
    fn skip_whitespace(&self) -> (usize, usize, usize) {
        let mut chars = self.remaining.chars();

        let mut consumed = 0;
        let mut line = self.line;
        let mut column = self.column;

        loop {
            match chars.next() {
                Some('/') if let Some('/') = chars.next() => {
                    consumed += 2;
                    column += 2;
                    loop {
                        match chars.next() {
                            Some('\n') => {
                                consumed += 1;
                                line += 1;
                                column = 0;
                                break;
                            }
                            Some(_) => {
                                consumed += 1;
                                column += 1;
                            }
                            None => break,
                        }
                    }
                }
                Some('\n') => {
                    line += 1;
                    column = 0;
                    consumed += 1;
                }
                Some(c) if c.is_ascii_whitespace() => {
                    column += 1;
                    consumed += 1;
                }
                _ => break,
            }
        }
        (consumed, line, column)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        let (consumed, line, column) = self.skip_whitespace();
        self.remaining = &self.remaining[consumed..];
        self.start += consumed;
        self.line = line;
        self.column = column;

        let mut chars = self.remaining.chars();

        let (consumed, token) = match chars.next()? {
            'a' => match chars.next() {
                Some('s') => self.make_keyword(chars, 2, "", Token::As),
                Some('c') => self.make_keyword(chars, 2, "tor", Token::Actor),
                Some(_) => self.make_identifier(chars, 2),
                None => self.make_identifier(chars, 1),
            },
            'b' => self.make_keyword(chars, 1, "oundary", Token::Boundary),
            'c' => match chars.next() {
                Some('o') => match chars.next() {
                    Some('l') => {
                        self.make_keyword(chars, 3, "lections", Token::Collections)
                    }
                    Some('m') => self.make_keyword(chars, 3, "ponent", Token::Component),
                    Some('n') => self.make_keyword(chars, 3, "trol", Token::Control),
                    Some(_) => self.make_identifier(chars, 3),
                    None => self.make_identifier(chars, 2),
                },
                Some(_) => self.make_identifier(chars, 2),
                None => self.make_identifier(chars, 1),
            },
            'd' => self.make_keyword(chars, 1, "atabase", Token::Database),
            'e' => self.make_keyword(chars, 1, "ntity", Token::Entity),
            'q' => self.make_keyword(chars, 1, "ueue", Token::Queue),
            c if c.is_ascii_alphabetic() => self.make_identifier(chars, 1),
            ';' => (
                1,
                Token::Semicolon(Span {
                    start: self.start,
                    length: 1,
                    line,
                    column,
                }),
            ),
            '"' => self.make_quoted_string(chars),
            _ => (
                1,
                Token::Invalid(
                    Span {
                        start: self.start,
                        length: 1,
                        line: self.line,
                        column: self.column,
                    },
                    &self.remaining[0..1],
                ),
            ),
        };

        self.remaining = &self.remaining[consumed..];
        self.start += consumed;
        self.column += consumed;

        Some(token)
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

    As(Span),

    Semicolon(Span),

    Invalid(Span, &'input str),
}

impl Token<'_> {
    pub fn lexer(input: &str) -> Lexer<'_> {
        Lexer::new(input)
    }

    pub fn span(&self) -> &Span {
        match self {
            Token::Component(span) => span,
            Token::Actor(span) => span,
            Token::Boundary(span) => span,
            Token::Control(span) => span,
            Token::Entity(span) => span,
            Token::Database(span) => span,
            Token::Collections(span) => span,
            Token::Queue(span) => span,
            Token::Identifier(span, _) => span,
            Token::QuotedString(span, _) => span,
            Token::As(span) => span,
            Token::Semicolon(span) => span,
            Token::Invalid(span, _) => span,
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
            Token::As(_) => write!(f, "As"),
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
        ($str:literal, $expected:expr) => {
            let mut lexer = Lexer::new($str);
            let token = lexer.next();
            assert_eq!(
                token,
                Some($expected),
                "expected {:?} but got {:?}",
                $expected,
                token
            );
        };
    }

    #[test]
    fn keyword_actor() {
        assert_token!(
            "actor",
            Token::Actor(Span {
                start: 0,
                length: 5,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn keyword_as() {
        assert_token!(
            "as",
            Token::As(Span {
                start: 0,
                length: 2,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn keyword_collections() {
        assert_token!(
            "collections",
            Token::Collections(Span {
                start: 0,
                length: 11,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn keyword_component() {
        assert_token!(
            "component",
            Token::Component(Span {
                start: 0,
                length: 9,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn keyword_control() {
        assert_token!(
            "control",
            Token::Control(Span {
                start: 0,
                length: 7,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn keyword_database() {
        assert_token!(
            "database",
            Token::Database(Span {
                start: 0,
                length: 8,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn keyword_entity() {
        assert_token!(
            "entity",
            Token::Entity(Span {
                start: 0,
                length: 6,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn keyword_queue() {
        assert_token!(
            "queue",
            Token::Queue(Span {
                start: 0,
                length: 5,
                line: 0,
                column: 0
            })
        );
    }

    #[test]
    fn identifier_a() {
        assert_token!(
            "a",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 1,
                    line: 0,
                    column: 0
                },
                "a"
            )
        );
    }

    #[test]
    fn identifier_ad() {
        assert_token!(
            "ad",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 2,
                    line: 0,
                    column: 0
                },
                "ad"
            )
        );
    }

    #[test]
    fn identifier_asd() {
        assert_token!(
            "asd",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 3,
                    line: 0,
                    column: 0
                },
                "asd"
            )
        );
    }

    #[test]
    fn identifier_collection() {
        assert_token!(
            "collection",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 10,
                    line: 0,
                    column: 0
                },
                "collection"
            )
        );
    }

    #[test]
    fn identifier_connection() {
        assert_token!(
            "connection",
            Token::Identifier(
                Span {
                    start: 0,
                    length: 10,
                    line: 0,
                    column: 0
                },
                "connection"
            )
        );
    }

    #[test]
    fn skip_whitespaces() {
        assert_token!(
            " \t\n ;",
            Token::Semicolon(Span {
                start: 4,
                length: 1,
                line: 1,
                column: 1
            })
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
            })
        );
    }

    #[test]
    fn quoted_string() {
        assert_token!(
            "\"string\"",
            Token::QuotedString(
                Span {
                    start: 0,
                    length: 8,
                    line: 0,
                    column: 0
                },
                Cow::Borrowed("string")
            )
        );
    }

    #[test]
    fn quoted_string_with_escaped_chars() {
        assert_token!(
            "\" \\\" \\n \\t \\x \"",
            Token::QuotedString(
                Span {
                    start: 0,
                    length: 15,
                    line: 0,
                    column: 0
                },
                Cow::Owned(" \" \n \t x ".to_string())
            )
        );
    }
}
