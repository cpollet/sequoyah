use crate::ast::{Event, Graph, Key, Note, Value};
use std::collections::HashSet;

type Result<T> = std::result::Result<T, String>;

impl Graph<'_> {
    pub fn check(self) -> Result<Self> {
        let mut components = HashSet::with_capacity(self.components.len());
        for component in &self.components {
            if !components.insert(component.identifier) {
                return Err(format!("Duplicate component {}", component.name));
            }
        }
        let components = components;

        for event in &self.events {
            match event {
                Event::Message(message) => {
                    if !components.contains(message.from) {
                        return Err(format!("Source component {} does not exist", message.from));
                    }
                    if !components.contains(message.to) {
                        return Err(format!(
                            "Destination component {} does not exist",
                            message.to
                        ));
                    }
                }
                Event::Note(note) => {
                    // todo check leftof / rightof component
                    let Note { attributes, .. } = note;

                    let mut leftof_rightof_seen = false;

                    for attribute in attributes {
                        match attribute.key {
                            Key::LeftOf => {
                                if leftof_rightof_seen {
                                    return Err("Duplicate leftof/rightof attribute".to_string());
                                }

                                leftof_rightof_seen = true;

                                match attribute.value {
                                    Value::Identifier(identifier) => {
                                        if !components.contains(identifier) {
                                            return Err(format!(
                                                "Identifier {identifier} not defined for leftof attribute",
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(
                                            "Identifier expected for leftof attribute".to_string()
                                        );
                                    }
                                }
                            }
                            Key::RightOf => {
                                if leftof_rightof_seen {
                                    return Err("Duplicate leftof/rightof attribute".to_string());
                                }

                                leftof_rightof_seen = true;

                                match attribute.value {
                                    Value::Identifier(identifier) => {
                                        if !components.contains(identifier) {
                                            return Err(format!(
                                                "Identifier {identifier} not defined for rightof attribute",
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(
                                            "Identifier expected for rightof attribute".to_string()
                                        );
                                    }
                                }
                            }
                            Key::Color => {}
                        }
                    }
                }
            }
        }

        let mut overlays = HashSet::with_capacity(self.overlays.len());
        for overlay in &self.overlays {
            if components.contains(overlay.identifier) {
                return Err(format!("Overlay {} shadows component", overlay.identifier));
            }

            if !overlays.insert(overlay.identifier) {
                return Err(format!("Duplicate overlay {}", overlay.identifier));
            }

            for component in &overlay.components {
                if !components.contains(component) {
                    return Err(format!(
                        "Component {} in overlay {} does not exist ",
                        component, overlay.identifier
                    ));
                }
            }
        }

        Ok(self)
    }
}
