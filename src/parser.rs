use crate::ast::{
    Activation, Attribute, Component, ComponentKind, Graph, Message, Overlay, Overlays, Value,
};
use crate::lexer::{PeekableLexer, Token};
use std::borrow::Cow;

type Lexer<'input> = PeekableLexer<'input>;
type TResult<T> = Result<T, String>;

impl Graph<'_> {
    pub fn parse(input: &str) -> TResult<Graph<'_>> {
        #[cfg(debug_assertions)]
        let mut lexer = Token::lexer(input).peekable();
        parse_graph(&mut lexer)
    }
}

fn parse_graph<'input>(lexer: &mut Lexer<'input>) -> TResult<Graph<'input>> {
    Ok(Graph {
        components: parse_components(lexer)?,
        overlays: parse_overlays(lexer)?,
        messages: parse_messages(lexer)?,
    })
}

fn parse_components<'input>(lexer: &mut Lexer<'input>) -> TResult<Vec<Component<'input>>> {
    let mut components = Vec::new();
    while let Some(component) = parse_component(lexer)? {
        components.push(component);
    }
    Ok(components)
}

fn parse_component<'input>(lexer: &mut Lexer<'input>) -> TResult<Option<Component<'input>>> {
    let component_kind = match lexer.peek() {
        Some(Token::Component(_)) => ComponentKind::Component,
        Some(Token::Actor(_)) => ComponentKind::Actor,
        Some(Token::Boundary(_)) => ComponentKind::Boundary,
        Some(Token::Control(_)) => ComponentKind::Control,
        Some(Token::Entity(_)) => ComponentKind::Entity,
        Some(Token::Database(_)) => ComponentKind::Database,
        Some(Token::Collections(_)) => ComponentKind::Collections,
        Some(Token::Queue(_)) => ComponentKind::Queue,
        _ => return Ok(None),
    };
    lexer.next();

    let (name, identifier) = parse_name_and_identifier(lexer)?;

    let attributes = parse_attributes(lexer)?;

    match lexer.next() {
        Some(Token::Semicolon(_)) => (),
        // todo add LBracket if not seen before
        Some(token) => {
            return Err(format!(
                "Expected Semicolon, got {token} at {span}",
                span = token.span()
            ));
        }
        None => return Err("Expected Semicolon, got end of input".into()),
    }

    Ok(Some(Component {
        kind: component_kind,
        name,
        identifier,
        attributes,
    }))
}

fn parse_name_and_identifier<'input>(
    lexer: &mut Lexer<'input>,
) -> TResult<(Cow<'input, str>, &'input str)> {
    let (name, identifier) = match lexer.next() {
        Some(Token::Identifier(_, identifier)) => (Cow::Borrowed(identifier), Some(identifier)),
        Some(Token::QuotedString(_, string)) => (string, None),
        Some(token) => {
            return Err(format!(
                "Expected name or identifier, got {token} at {span}",
                span = token.span()
            ));
        }
        None => return Err("Expected name or identifier, got end of input".into()),
    };

    let identifier = if let Some(identifier) = identifier {
        identifier
    } else {
        match lexer.next() {
            Some(Token::As(_)) => (),
            Some(token) => {
                return Err(format!(
                    "Expected 'as', got {token} at {span}",
                    span = token.span()
                ));
            }
            None => return Err("Expected 'as', got end of input".into()),
        }

        match lexer.next() {
            Some(Token::Identifier(_, identifier)) => identifier,
            Some(token) => {
                return Err(format!(
                    "Expected identifier, got {token} at {span}",
                    span = token.span()
                ));
            }
            None => return Err("Expected identifier, got end of input".into()),
        }
    };

    Ok((name, identifier))
}

fn parse_messages<'input>(lexer: &mut Lexer<'input>) -> TResult<Vec<Message<'input>>> {
    let mut messages = Vec::new();
    while let Some(message) = parse_message(lexer)? {
        messages.push(message);
    }
    Ok(messages)
}

fn parse_message<'input>(lexer: &mut Lexer<'input>) -> TResult<Option<Message<'input>>> {
    let Some(from) = parse_identifier(lexer)? else {
        return Ok(None);
    };

    if !lexer.skip_whitespace() {
        let (line, column) = lexer.location();
        return Err(format!(
            "Expected whitespace at {}:{}",
            line + 1,
            column + 1
        ));
    }

    let arrow = match lexer.next() {
        Some(Token::Arrow(_, arrow)) => arrow,
        Some(token) => {
            return Err(format!(
                "Expected arrow, got {token} at {span}",
                span = token.span()
            ));
        }
        None => return Err("Expected arrow, got end of input".into()),
    };

    if !lexer.skip_whitespace() {
        let (line, column) = lexer.location();
        return Err(format!(
            "Expected whitespace at {}:{}",
            line + 1,
            column + 1
        ));
    }

    let to = match parse_identifier(lexer) {
        Ok(Some(i)) => i,
        Ok(None) => return Err("Expected identifier, got end of input".into()),
        Err(e) => return Err(e),
    };

    let attributes = parse_attributes(lexer)?;
    let message = parse_message_text(lexer);
    let activation = parse_activation(lexer)?;

    Ok(Some(Message {
        from,
        arrow: arrow.clone(),
        to,
        attributes,
        text: message,
        activation,
    }))
}

fn parse_identifier<'input>(lexer: &mut Lexer<'input>) -> TResult<Option<&'input str>> {
    match lexer.next() {
        None => Ok(None),
        Some(Token::Identifier(_, identifier)) => Ok(Some(identifier)),
        Some(token) => Err(format!(
            "Expected identifier, got {token} at {span}",
            span = token.span()
        )),
    }
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
            match parse_message(lexer)? {
                Some(message) => messages.push(message),
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

fn parse_overlays<'input>(lexer: &mut Lexer<'input>) -> TResult<Overlays<'input>> {
    let mut overlays = Overlays::new();
    while let Some(overlay) = parse_overlay(lexer)? {
        overlays.push(overlay);
    }
    Ok(overlays)
}

fn parse_overlay<'input>(lexer: &mut Lexer<'input>) -> TResult<Option<Overlay<'input>>> {
    if lexer
        .next_if(|token| matches!(token, Token::Overlay(_)))
        .is_none()
    {
        return Ok(None);
    }

    let kind = match lexer.next() {
        Some(Token::Component(_)) => ComponentKind::Component,
        Some(Token::Actor(_)) => ComponentKind::Actor,
        Some(Token::Boundary(_)) => ComponentKind::Boundary,
        Some(Token::Control(_)) => ComponentKind::Control,
        Some(Token::Entity(_)) => ComponentKind::Entity,
        Some(Token::Database(_)) => ComponentKind::Database,
        Some(Token::Collections(_)) => ComponentKind::Collections,
        Some(Token::Queue(_)) => ComponentKind::Queue,
        Some(token) => {
            return Err(format!(
                "Expected component kind, got {token} at {span}",
                span = token.span()
            ));
        }
        None => return Err("Expected component kind, got end of input".into()),
    };

    let (name, identifier) = parse_name_and_identifier(lexer)?;

    let attributes = parse_attributes(lexer)?;

    match lexer.next() {
        Some(Token::Colon(_)) => (),
        // todo add LBracket if not seen before
        Some(token) => {
            return Err(format!(
                "Expected Colon, got {token} at {span}",
                span = token.span()
            ));
        }
        None => return Err("Expected Colon, got end of input".into()),
    }

    let mut components = Vec::new();
    loop {
        if lexer
            .next_if(|token| matches!(token, Token::Semicolon(_)))
            .is_some()
        {
            break;
        }
        match parse_identifier(lexer)? {
            Some(identifier) => components.push(identifier),
            None => return Err("Expected Semicolon or identifier, got end of input".into()),
        }
    }

    Ok(Some(Overlay {
        kind,
        name,
        identifier,
        components,
        attributes,
    }))
}
