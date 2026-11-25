use crate::ir::Shape;
use crate::{ast, ir};
use indexmap::IndexMap;
use std::collections::HashMap;

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
        let mut attributes = into_map(value.attributes);

        Ok(ir::Component {
            kind: value.kind,
            name: value.name,
            identifier: value.identifier,
            color: no_other(&mut attributes, extract_color)?,
        })
    }
}

impl<'input> TryFrom<ast::Event<'input>> for ir::Event<'input> {
    type Error = Error;

    fn try_from(value: ast::Event<'input>) -> std::result::Result<Self, Self::Error> {
        match value {
            ast::Event::Message(message) => ir::Message::try_from(message).map(ir::Event::Message),
            ast::Event::Note(note) => ir::Note::try_from(note).map(ir::Event::Note),
            ast::Event::Divider(divider) => ir::Divider::try_from(divider).map(ir::Event::Divider),
            ast::Event::Later(later) => ir::Later::try_from(later).map(ir::Event::Later),
        }
    }
}

impl<'input> TryFrom<ast::Message<'input>> for ir::Message<'input> {
    type Error = Error;

    fn try_from(value: ast::Message<'input>) -> std::result::Result<Self, Self::Error> {
        let mut attributes = into_map(value.attributes);

        Ok(ir::Message {
            from: value.from,
            arrow: value.arrow,
            to: value.to,
            text: value.text,
            color: no_other(&mut attributes, extract_color)?,
            activation: value.activation.map(ir::Activation::try_from).transpose()?,
        })
    }
}

impl<'input> TryFrom<ast::Activation<'input>> for ir::Activation<'input> {
    type Error = Error;

    fn try_from(value: ast::Activation<'input>) -> std::result::Result<Self, Self::Error> {
        let ret_val = match value.ret_val {
            Some((attributes, text)) => {
                let mut attributes = into_map(attributes);

                Some(ir::RetVal {
                    color: no_other(&mut attributes, extract_color)?,
                    text,
                })
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
        let mut attributes = into_map(value.attributes);

        Ok(ir::Note {
            text: value.text,
            color: extract_color(&mut attributes)?,
            shape: extract_shape(&mut attributes)?,
            position: no_other(&mut attributes, extract_position)?,
        })
    }
}

impl<'input> TryFrom<ast::Overlay<'input>> for ir::Overlay<'input> {
    type Error = Error;

    fn try_from(value: ast::Overlay<'input>) -> std::result::Result<Self, Self::Error> {
        let mut attributes = into_map(value.attributes);

        Ok(ir::Overlay {
            kind: value.kind,
            name: value.name,
            identifier: value.identifier,
            components: value.components,
            color: no_other(&mut attributes, extract_color)?,
        })
    }
}

impl<'input> TryFrom<ast::Divider<'input>> for ir::Divider<'input> {
    type Error = Error;

    fn try_from(value: ast::Divider<'input>) -> std::result::Result<Self, Self::Error> {
        Ok(ir::Divider { text: value.text })
    }
}

impl<'input> TryFrom<ast::Later<'input>> for ir::Later<'input> {
    type Error = Error;

    fn try_from(value: ast::Later<'input>) -> std::result::Result<Self, Self::Error> {
        Ok(ir::Later { text: value.text })
    }
}

fn into_map(attributes: ast::Attributes) -> HashMap<ast::Key, ast::Value> {
    attributes
        .into_iter()
        .map(|attribute| (attribute.key, attribute.value))
        .collect()
}

fn no_other<'input, F, T>(
    attributes: &mut HashMap<ast::Key, ast::Value<'input>>,
    extractor: F,
) -> Result<T>
where
    F: Fn(&mut HashMap<ast::Key, ast::Value<'input>>) -> Result<T> + 'input,
{
    let val = extractor(attributes)?;

    if let Some(key) = attributes.keys().next() {
        return Err(format!("Invalid attribute {key}"));
    }

    Ok(val)
}

fn extract_color<'input>(
    attributes: &mut HashMap<ast::Key, ast::Value<'input>>,
) -> Result<Option<&'input str>> {
    match attributes.remove("color") {
        Some(value) => match value {
            ast::Value::Identifier(identifier) => match crate::colors::COLORS.get(identifier) {
                Some(color) => Ok(Some(color)),
                None => Err(format!("Invalid color {identifier}")),
            },
            ast::Value::HexColor(hcolor) => Ok(Some(hcolor)),
            ast::Value::Quoted(_) => Err("Attribute color cannot be a string".to_string()),
            ast::Value::Boolean(_) => Err("Attribute color cannot be a boolean".to_string()),
        },
        None => Ok(None),
    }
}

fn extract_position<'input>(
    attributes: &mut HashMap<ast::Key, ast::Value<'input>>,
) -> Result<Option<ir::Position<'input>>> {
    let mut position = None;

    for (attribute, mapper) in [
        ("leftof", Box::new(|v| ir::Position::LeftOf(v))),
        ("rightof", Box::new(|v| ir::Position::RightOf(v))),
    ] as [(&str, Box<dyn Fn(&str) -> ir::Position>); 2]
    {
        let new_position = match attributes.remove(attribute) {
            Some(value) => match value {
                ast::Value::Identifier(identifier) => Some(mapper(identifier)),
                ast::Value::Quoted(_) => {
                    return Err(format!("Attribute {attribute} cannot be a string"));
                }
                ast::Value::HexColor(_) => {
                    return Err(format!("Attribute {attribute} cannot be an hex color"));
                }
                ast::Value::Boolean(_) => {
                    return Err(format!("Attribute {attribute} cannot be a boolean"));
                }
            },
            None => None,
        };

        if let Some(new_position) = new_position {
            if position.is_some() {
                return Err(format!(
                    "Cannot set {attribute}, another position attribute already defined"
                ));
            }
            position = Some(new_position);
        }
    }

    for (attribute, ir_position) in [("left", ir::Position::Left), ("right", ir::Position::Right)] {
        let new_position = match attributes.remove(attribute) {
            Some(value) => match value {
                ast::Value::Identifier(_) => {
                    return Err(format!("Attribute {attribute} cannot be an identifier"));
                }
                ast::Value::Quoted(_) => {
                    return Err(format!("Attribute {attribute} cannot be a string"));
                }
                ast::Value::HexColor(_) => {
                    return Err(format!("Attribute {attribute} cannot be an hex color"));
                }
                ast::Value::Boolean(_) => Some(ir_position),
            },
            None => None,
        };

        if let Some(new_position) = new_position {
            if position.is_some() {
                return Err(format!(
                    "Cannot set {attribute}, another position attribute already defined"
                ));
            }
            position = Some(new_position);
        }
    }

    Ok(position)
}

fn extract_shape(attributes: &mut HashMap<ast::Key, ast::Value>) -> Result<Option<Shape>> {
    if let Some(shape) = attributes.remove("shape") {
        match shape {
            ast::Value::Identifier("rectangle") => Ok(Some(Shape::Rectangle)),
            ast::Value::Identifier("hexagonal") => Ok(Some(Shape::Hexagonal)),
            ast::Value::Identifier("default") => Ok(Some(Shape::Default)),
            ast::Value::Identifier(identifier) => {
                Err(format!("Invalid value '{identifier}' for shape attribute"))
            }
            ast::Value::Quoted(_) => Err("Attribute shape cannot be a string".to_string()),
            ast::Value::HexColor(_) => Err("Attribute shape cannot be an hex color".to_string()),
            ast::Value::Boolean(_) => Err("Attribute shape cannot be a boolean".to_string()),
        }
    } else {
        Ok(None)
    }
}
