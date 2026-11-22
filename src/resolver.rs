use crate::ast::{
    Activation, Component, Components, Event, Graph, Key, Message, Note, Overlay, Overlays, Value,
};
use std::collections::{HashMap, HashSet};

type Result<T> = std::result::Result<T, String>;
impl Graph<'_> {
    pub fn resolve(self, expanded_overlays: &HashSet<&str>) -> Result<Self> {
        if expanded_overlays.contains("*") {
            return Ok(self);
        }

        let Graph {
            components,
            overlays,
            events,
        } = self;

        let (components, component_map) =
            resolve_components(overlays, components, expanded_overlays)?;

        let events = events
            .into_iter()
            .filter_map(|event| resolve_event(event, &component_map))
            .collect::<Vec<_>>();

        Ok(Self {
            components,
            overlays: Vec::default(),
            events,
        })
    }
}

fn resolve_components<'input>(
    overlays: Overlays<'input>,
    components: Components<'input>,
    expanded_overlays: &HashSet<&str>,
) -> Result<(Vec<Component<'input>>, HashMap<&'input str, &'input str>)> {
    let overlays_by_component = overlays
        .iter()
        .flat_map(|overlay| {
            overlay
                .components
                .iter()
                .map(|component| (overlay.identifier, *component))
        })
        .fold(HashMap::new(), |mut acc, (overlay, component)| {
            acc.entry(component).or_insert(Vec::new()).push(overlay);
            acc
        });

    let mut overlays_by_identifier = overlays
        .into_iter()
        .map(|o| (o.identifier, o))
        .collect::<HashMap<_, _>>();

    let mut displayed_components = Vec::with_capacity(components.len());
    let mut displayed_overlays = HashSet::new();
    let mut component_map = HashMap::new();

    for component in components {
        match overlays_by_component.get(component.identifier) {
            None => displayed_components.push(component),
            Some(overlays) if overlays.iter().any(|o| expanded_overlays.contains(o)) => {
                displayed_components.push(component);
            }
            Some(overlays) if overlays.len() != 1 => {
                return Err(format!(
                    "{} overlays found for component {}",
                    overlays.len(),
                    component.identifier
                ));
            }
            Some(overlays) => {
                let overlay_identifier = overlays[0];
                component_map.insert(component.identifier, overlay_identifier);

                if displayed_overlays.contains(overlay_identifier) {
                    continue;
                }

                match overlays_by_identifier.remove(overlay_identifier) {
                    Some(overlay) => {
                        displayed_overlays.insert(overlay.identifier);
                        displayed_components.push(Component::from(overlay));
                    }
                    None => {
                        return Err(format!(
                            "No overlays found for component {}",
                            component.identifier
                        ));
                    }
                }
            }
        }
    }

    Ok((displayed_components, component_map))
}

fn resolve_event<'input>(
    event: Event<'input>,
    component_map: &HashMap<&'input str, &'input str>,
) -> Option<Event<'input>> {
    match event {
        Event::Message(message) => resolve_message(message, component_map).map(Event::Message),
        Event::Note(note) => resolve_note(note, component_map).map(Event::Note),
    }
}

fn resolve_message<'input>(
    mut message: Message<'input>,
    component_map: &HashMap<&'input str, &'input str>,
) -> Option<Message<'input>> {
    let from_hidden = component_map.get(message.from).is_some();
    let to_hidden = component_map.get(message.from).is_some();

    if from_hidden && to_hidden {
        None
    } else {
        let activation = match message.activation.take() {
            Some(activation) => Some(Activation {
                events: activation
                    .events
                    .into_iter()
                    .filter_map(|event| resolve_event(event, component_map))
                    .collect::<Vec<_>>(),
                ret_val: activation.ret_val,
            }),
            None => None,
        };

        Some(Message {
            from: component_map.get(&message.from).unwrap_or(&message.from),
            to: component_map.get(&message.to).unwrap_or(&message.to),
            activation,
            ..message
        })
    }
}

fn resolve_note<'input>(
    note: Note<'input>,
    component_map: &HashMap<&'input str, &'input str>,
) -> Option<Note<'input>> {
    if let Some(ref_component) = note.attributes.iter().find_map(|attr| match attr.key {
        Key::Color => None,
        Key::LeftOf | Key::RightOf => Some(match attr.value {
            Value::Identifier(identifier) => identifier,
            Value::Quoted(_) | Value::HexColor(_) => unreachable!(),
        }),
    }) && component_map.contains_key(ref_component)
    {
        return None;
    }

    Some(note)
}

impl<'input> From<Overlay<'input>> for Component<'input> {
    fn from(value: Overlay<'input>) -> Self {
        Component {
            kind: value.kind,
            name: value.name,
            identifier: value.identifier,
            attributes: value.attributes,
        }
    }
}
