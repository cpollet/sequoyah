use crate::ast::Graph;
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

        for message in &self.messages {
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
