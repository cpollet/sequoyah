use crate::ast::{Component, ComponentKind, Graph};
use crate::lexer;
use crate::lexer::Token;
use std::borrow::Cow;
use std::iter::Peekable;

type Lexer<'input> = Peekable<lexer::Lexer<'input>>;
type TResult<T> = Result<T, String>;

pub fn parse(input: &str) -> TResult<Graph<'_>> {
    let mut lexer = Token::lexer(input).peekable();
    parse_graph(&mut lexer)
}

fn parse_graph<'input>(lexer: &mut Lexer<'input>) -> TResult<Graph<'input>> {
    Ok(Graph {
        components: parse_components(lexer)?,
        messages: vec![],
    })
}

fn parse_components<'input>(lexer: &mut Lexer<'input>) -> TResult<Vec<Component<'input>>> {
    let mut components = Vec::new();
    while let Some(component) = parse_component(lexer) {
        components.push(component?);
    }
    Ok(components)
}

fn parse_component<'input>(lexer: &mut Lexer<'input>) -> Option<TResult<Component<'input>>> {
    let component_kind = match lexer.peek() {
        Some(Token::Component(_)) => ComponentKind::Component,
        Some(Token::Actor(_)) => ComponentKind::Actor,
        Some(Token::Boundary(_)) => ComponentKind::Boundary,
        Some(Token::Control(_)) => ComponentKind::Control,
        Some(Token::Entity(_)) => ComponentKind::Entity,
        Some(Token::Database(_)) => ComponentKind::Database,
        Some(Token::Collections(_)) => ComponentKind::Collections,
        Some(Token::Queue(_)) => ComponentKind::Queue,
        Some(_) => return None,
        None => return None,
    };
    lexer.next();

    let (name, identifier) = match lexer.next() {
        Some(Token::Identifier(_, identifier)) => (Cow::Borrowed(identifier), Some(identifier)),
        Some(Token::QuotedString(_, string)) => (string, None),
        Some(token) => {
            return Some(Err(format!(
                "Expected name or identifier, got {token} at {span}",
                span = token.span()
            )));
        }
        None => return Some(Err("Expected name or identifier, got end of input".into())),
    };

    let identifier = match identifier {
        Some(identifier) => identifier,
        None => {
            match lexer.next() {
                Some(Token::As(_)) => (),
                Some(token) => {
                    return Some(Err(format!(
                        "Expected 'as', got {token} as {span}",
                        span = token.span()
                    )));
                }
                None => return Some(Err("Expected 'as', got end of input".into())),
            }

            match lexer.next() {
                Some(Token::Identifier(_, identifier)) => identifier,
                Some(token) => {
                    return Some(Err(format!(
                        "Expected identifier, got {token} at {span}",
                        span = token.span()
                    )));
                }
                None => return Some(Err("Expected identifier, got end of input".into())),
            }
        }
    };

    match lexer.next() {
        Some(Token::Semicolon(_)) => (),
        Some(token) => {
            return Some(Err(format!(
                "Expected ';', got {token} at {span}",
                span = token.span()
            )));
        }
        None => return Some(Err("Expected ';', got end of input".into())),
    }

    Some(Ok(Component {
        kind: component_kind,
        name,
        alias: identifier,
    }))
}
