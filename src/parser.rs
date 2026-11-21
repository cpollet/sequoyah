use crate::ast::{Activation, Attribute, Component, ComponentKind, Graph, Message, Value};
use crate::lexer::{PeekableLexer, Token};
use std::borrow::Cow;

type Lexer<'input> = PeekableLexer<'input>;
type TResult<T> = Result<T, String>;

pub fn parse(input: &str) -> TResult<Graph<'_>> {
    #[cfg(debug_assertions)]
    let mut lexer = Token::lexer(input).peekable();
    parse_graph(&mut lexer)
}

fn parse_graph<'input>(lexer: &mut Lexer<'input>) -> TResult<Graph<'input>> {
    Ok(Graph {
        components: parse_components(lexer)?,
        messages: parse_messages(lexer)?,
    })
}

fn parse_components<'input>(lexer: &mut Lexer<'input>) -> TResult<Vec<Component<'input>>> {
    let mut components = Vec::new();
    while let Some(component) = parse_component(lexer) {
        components.push(component?);
    }
    Ok(components)
}

// todo refactor: should be TResult<Option, ..>>
fn parse_component<'input>(lexer: &mut Lexer<'input>) -> Option<TResult<Component<'input>>> {
    let component_kind = match lexer.peek()? {
        Token::Component(_) => ComponentKind::Component,
        Token::Actor(_) => ComponentKind::Actor,
        Token::Boundary(_) => ComponentKind::Boundary,
        Token::Control(_) => ComponentKind::Control,
        Token::Entity(_) => ComponentKind::Entity,
        Token::Database(_) => ComponentKind::Database,
        Token::Collections(_) => ComponentKind::Collections,
        Token::Queue(_) => ComponentKind::Queue,
        _ => return None,
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

    let identifier = if let Some(identifier) = identifier {
        identifier
    } else {
        match lexer.next() {
            Some(Token::As(_)) => (),
            Some(token) => {
                return Some(Err(format!(
                    "Expected 'as', got {token} at {span}",
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

fn parse_messages<'input>(lexer: &mut Lexer<'input>) -> TResult<Vec<Message<'input>>> {
    let mut messages = Vec::new();
    while let Some(message) = parse_message(lexer) {
        messages.push(message?);
    }
    Ok(messages)
}

// todo refactor: should be TResult<Option, ..>>
fn parse_message<'input>(lexer: &mut Lexer<'input>) -> Option<TResult<Message<'input>>> {
    let from = match parse_identifier(lexer)? {
        Ok(i) => i,
        Err(e) => return Some(Err(e)),
    };

    if !lexer.skip_whitespace() {
        // todo report span
        return Some(Err("Expected whitespace".to_string()));
    }

    let arrow = match lexer.next() {
        Some(Token::Arrow(_, arrow)) => arrow,
        Some(token) => {
            return Some(Err(format!(
                "Expected arrow, got {token} at {span}",
                span = token.span()
            )));
        }
        None => return Some(Err("Expected arrow, got end of input".into())),
    };

    if !lexer.skip_whitespace() {
        // todo report span
        return Some(Err("Expected whitespace".to_string()));
    }

    let to = match parse_identifier(lexer) {
        Some(Ok(i)) => i,
        Some(Err(e)) => return Some(Err(e)),
        None => return Some(Err("Expected identifier, got end of input".into())),
    };

    let attributes = match parse_attributes(lexer) {
        Ok(attributes) => attributes,
        Err(e) => return Some(Err(e)),
    };

    let message = parse_message_text(lexer);

    let activation = match parse_activation(lexer) {
        Ok(activation) => activation,
        Err(e) => return Some(Err(e)),
    };

    Some(Ok(Message {
        from,
        arrow: arrow.clone(),
        to,
        attributes,
        text: message,
        activation,
    }))
}

// todo refactor: should be TResult<Option, ..>>
fn parse_identifier<'input>(lexer: &mut Lexer<'input>) -> Option<TResult<&'input str>> {
    Some(match lexer.next()? {
        Token::Identifier(_, identifier) => Ok(identifier),
        token => Err(format!(
            "Expected identifier, got {token} at {span}",
            span = token.span()
        )),
    })
}

fn parse_attributes<'input>(lexer: &mut Lexer<'input>) -> TResult<Vec<Attribute<'input>>> {
    let mut attributes = Vec::new();
    if lexer
        .next_if(|token| matches!(token, Token::LBracket(_)))
        .is_some()
    {
        let mut can_parse_new_attribute = true;
        loop {
            if let Some(token) = lexer.next() {
                let key = match token {
                    Token::RBracket(_) => break,
                    Token::Identifier(_, identifier) if can_parse_new_attribute => identifier,
                    token => {
                        if can_parse_new_attribute {
                            return Err(format!(
                                "Expected identifier, got {token} at {span}",
                                span = token.span()
                            ));
                        }
                        return Err(format!(
                            "Expected Comma, got {token} at {span}",
                            span = token.span()
                        ));
                    }
                };

                match lexer.next() {
                    Some(Token::Equal(_)) => (),
                    Some(token) => {
                        return Err(format!(
                            "Expected Equal, got {token} at {span}",
                            span = token.span()
                        ));
                    }
                    None => return Err("Expected Equal, got end of input".into()),
                }

                let value = match lexer.next() {
                    Some(Token::QuotedString(_, string)) => Value::Quoted(string),
                    Some(Token::HexColor(_, color)) => Value::HexColor(color),
                    Some(Token::Identifier(_, identifier)) => Value::Identifier(identifier),
                    Some(token) => {
                        return Err(format!(
                            "Expected Color, got {token} at {span}",
                            span = token.span()
                        ));
                    }
                    None => return Err("Expected Color, got end of input".into()),
                };

                can_parse_new_attribute = lexer
                    .next_if(|token| matches!(token, Token::Comma(_)))
                    .is_some();

                attributes.push(Attribute { key, value });
            } else {
                if can_parse_new_attribute {
                    return Err("Expected identifier, got end of input".to_string());
                }
                return Err("Expected Comma, got end of input".to_string());
            }
        }
    }
    Ok(attributes)
}

fn parse_message_text<'input>(lexer: &mut Lexer<'input>) -> Option<Cow<'input, str>> {
    lexer
        .next_if(|token| matches!(token, Token::Colon(_)))
        .map(|_| lexer.unquoted_string())
        .map(|t| match t {
            Token::UnquotedString(_, string) => string,
            _ => unreachable!(),
        })
}

fn parse_activation<'input>(lexer: &mut Lexer<'input>) -> TResult<Option<Activation<'input>>> {
    if lexer
        .next_if(|token| matches!(token, Token::LCurly(_)))
        .is_some()
    {
        let mut messages = Vec::new();
        let ret_val = loop {
            if lexer
                .next_if(|token| matches!(token, Token::RCurly(_)))
                .is_some()
            {
                break None;
            }
            if lexer
                .next_if(|token| matches!(token, Token::Return(_)))
                .is_some()
            {
                let attributes = parse_attributes(lexer)?;

                let text = parse_message_text(lexer);

                match lexer.next() {
                    Some(Token::RCurly(_)) => break text.map(|t| (attributes, t)),
                    Some(token) => {
                        // todo: add RBracket and Colon of not seen before ...
                        return Err(format!(
                            "Expected RCurly, got {token} at {span}",
                            span = token.span()
                        ));
                    }
                    // todo: add RBracket and Colon of not seen before ...
                    None => return Err("Expected RCurly, got end of input".into()),
                }
            }
            match parse_message(lexer) {
                Some(Err(e)) => return Err(e),
                Some(Ok(message)) => messages.push(message),
                None => {
                    return Err(
                        "Expected RCurly, Return or identifier, got end of input".to_string()
                    );
                }
            }
        };

        Ok(Some(Activation { messages, ret_val }))
    } else {
        Ok(None)
    }
}
