use crate::{ast, ir};
use indexmap::IndexMap;

type Error = String;
type Result<T> = std::result::Result<T, Error>;

impl<'input> ast::Graph<'input> {
    pub fn check(self) -> Result<ir::Graph<'input>> {
        let mut components = IndexMap::with_capacity(self.components.len());
        for component in self.components {
            if components.contains_key(component.identifier) {
                return Err(format!("Duplicate component {}", component.name));
            }

            components.insert(component.identifier, ir::Component::try_from(component)?);
        }
        let components = components;

        let mut events = Vec::with_capacity(self.events.len());
        for event in self.events {
            // todo check that all ref. components exist
            events.push(ir::Event::try_from(event)?);
        }

        let mut overlays = IndexMap::with_capacity(self.overlays.len());
        for overlay in self.overlays {
            if components.contains_key(overlay.identifier) {
                return Err(format!("Overlay {} shadows component", overlay.identifier));
            }
            if overlays.contains_key(overlay.identifier) {
                return Err(format!("Duplicate overlay {}", overlay.identifier));
            }
            for component in &overlay.components {
                if !components.contains_key(component) {
                    return Err(format!(
                        "Component {} in overlay {} does not exist ",
                        component, overlay.identifier
                    ));
                }
            }

            overlays.insert(overlay.identifier, ir::Overlay::try_from(overlay)?);
        }

        Ok(ir::Graph {
            components: components.into_values().collect(),
            overlays: overlays.into_values().collect(),
            events,
        })
    }
}

impl<'input> TryFrom<ast::Component<'input>> for ir::Component<'input> {
    type Error = Error;

    fn try_from(value: ast::Component<'input>) -> std::result::Result<Self, Self::Error> {
        let mut color = None;

        for attribute in value.attributes {
            match &attribute.key {
                ast::Key::Color => match attribute.value {
                    // todo error reporting on invalid quoted color and on invalid color name (identifier)
                    // todo parse quoted color as either an identifier or an hexcolor
                    ast::Value::Identifier(identifier) => {
                        color = crate::colors::COLORS.get(identifier).copied();
                    }
                    ast::Value::Quoted(_) => {
                        return Err("Attribute color cannot be a string".to_string());
                    }
                    ast::Value::HexColor(hcolor) => color = Some(hcolor),
                },
                ast::Key::LeftOf => {
                    return Err("Attribute leftof not supported on component".to_string());
                }
                ast::Key::RightOf => {
                    return Err("Attribute rightof not supported on component".to_string());
                }
            }
        }

        Ok(ir::Component {
            kind: value.kind,
            name: value.name,
            identifier: value.identifier,
            color,
        })
    }
}

impl<'input> TryFrom<ast::Event<'input>> for ir::Event<'input> {
    type Error = Error;

    fn try_from(value: ast::Event<'input>) -> std::result::Result<Self, Self::Error> {
        match value {
            ast::Event::Message(message) => ir::Message::try_from(message).map(ir::Event::Message),
            ast::Event::Note(note) => ir::Note::try_from(note).map(ir::Event::Note),
        }
    }
}

impl<'input> TryFrom<ast::Message<'input>> for ir::Message<'input> {
    type Error = Error;

    fn try_from(value: ast::Message<'input>) -> std::result::Result<Self, Self::Error> {
        let mut color = None;

        for attribute in value.attributes {
            match &attribute.key {
                ast::Key::Color => match attribute.value {
                    ast::Value::Identifier(identifier) => {
                        color = crate::colors::COLORS.get(identifier).copied();
                    }
                    ast::Value::Quoted(_) => {
                        return Err("Attribute color cannot be a string".to_string());
                    }
                    ast::Value::HexColor(hcolor) => color = Some(hcolor),
                },
                ast::Key::LeftOf => {
                    return Err("Attribute leftof not supported on component".to_string());
                }
                ast::Key::RightOf => {
                    return Err("Attribute rightof not supported on component".to_string());
                }
            }
        }

        Ok(ir::Message {
            from: value.from,
            arrow: value.arrow,
            to: value.to,
            text: value.text,
            color,
            activation: value.activation.map(ir::Activation::try_from).transpose()?,
        })
    }
}

impl<'input> TryFrom<ast::Activation<'input>> for ir::Activation<'input> {
    type Error = Error;

    fn try_from(value: ast::Activation<'input>) -> std::result::Result<Self, Self::Error> {
        let ret_val = match value.ret_val {
            Some((attributes, text)) => {
                let mut color = None;

                for attribute in attributes {
                    match &attribute.key {
                        ast::Key::Color => match attribute.value {
                            ast::Value::Identifier(identifier) => {
                                color = crate::colors::COLORS.get(identifier).copied();
                            }
                            ast::Value::Quoted(_) => {
                                return Err("Attribute color cannot be a string".to_string());
                            }
                            ast::Value::HexColor(hcolor) => color = Some(hcolor),
                        },
                        ast::Key::LeftOf => {
                            return Err("Attribute leftof not supported on component".to_string());
                        }
                        ast::Key::RightOf => {
                            return Err("Attribute rightof not supported on component".to_string());
                        }
                    }
                }
                Some(ir::RetVal { color, text })
            }
            None => None,
        };

        let mut events = Vec::with_capacity(value.events.len());
        for event in value.events {
            events.push(ir::Event::try_from(event)?);
        }

        Ok(ir::Activation { events, ret_val })
    }
}

impl<'input> TryFrom<ast::Note<'input>> for ir::Note<'input> {
    type Error = Error;

    fn try_from(value: ast::Note<'input>) -> std::result::Result<Self, Self::Error> {
        let mut color = None;
        let mut position = None;

        for attribute in value.attributes {
            match &attribute.key {
                ast::Key::Color => match attribute.value {
                    ast::Value::Identifier(identifier) => {
                        color = crate::colors::COLORS.get(identifier).copied();
                    }
                    ast::Value::Quoted(_) => {
                        return Err("Attribute color cannot be a string".to_string());
                    }
                    ast::Value::HexColor(hcolor) => color = Some(hcolor),
                },
                ast::Key::LeftOf => match attribute.value {
                    ast::Value::Identifier(identifier) => {
                        position = Some(ir::Position::LeftOf(identifier));
                    }
                    ast::Value::Quoted(_) => {
                        return Err("Attribute leftof cannot be a string".to_string());
                    }
                    ast::Value::HexColor(_) => {
                        return Err("Attribute leftof cannot be an hexcolor".to_string());
                    }
                },
                ast::Key::RightOf => match attribute.value {
                    ast::Value::Identifier(identifier) => {
                        position = Some(ir::Position::RightOf(identifier));
                    }
                    ast::Value::Quoted(_) => {
                        return Err("Attribute rightof cannot be a string".to_string());
                    }
                    ast::Value::HexColor(_) => {
                        return Err("Attribute rightof cannot be an hexcolor".to_string());
                    }
                },
            }
        }

        Ok(ir::Note {
            text: value.text,
            color,
            position,
        })
    }
}

impl<'input> TryFrom<ast::Overlay<'input>> for ir::Overlay<'input> {
    type Error = Error;

    fn try_from(value: ast::Overlay<'input>) -> std::result::Result<Self, Self::Error> {
        let mut color = None;

        for attribute in value.attributes {
            match &attribute.key {
                ast::Key::Color => match attribute.value {
                    ast::Value::Identifier(identifier) => {
                        color = crate::colors::COLORS.get(identifier).copied();
                    }
                    ast::Value::Quoted(_) => {
                        return Err("Attribute color cannot be a string".to_string());
                    }
                    ast::Value::HexColor(hcolor) => color = Some(hcolor),
                },
                ast::Key::LeftOf => {
                    return Err("Attribute leftof not supported on overlay".to_string());
                }
                ast::Key::RightOf => {
                    return Err("Attribute rightof not supported on overlay".to_string());
                }
            }
        }

        Ok(ir::Overlay {
            kind: value.kind,
            name: value.name,
            identifier: value.identifier,
            components: value.components,
            color,
        })
    }
}
