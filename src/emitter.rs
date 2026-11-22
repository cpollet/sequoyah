use crate::ast::Graph;

trait Emit {
    fn emit<E>(&self, emitter: E)
    where
        E: Emitter;
}

impl Emit for Graph<'_> {
    fn emit<E>(&self, emitter: E)
    where
        E: Emitter,
    {
        emitter.emit_graph(self);
    }
}

trait Emitter {
    fn emit_graph(self, graph: &Graph<'_>);
}

pub mod plantuml {
    use crate::ast::{
        Arrow, ArrowBody, ArrowDirection, ArrowHead, Attribute, Component, ComponentKind, Event,
        Graph, Key, Message, Note, Value,
    };
    use crate::emitter::{Emit, Emitter};
    use std::fmt::Write;

    pub fn emit(graph: &Graph) -> String {
        let mut plant_uml = PlantUml::new();
        graph.emit(&mut plant_uml);
        plant_uml.output
    }

    struct PlantUml {
        output: String,
    }

    impl PlantUml {
        fn new() -> Self {
            Self {
                output: String::new(),
            }
        }

        fn emit_component(&mut self, component: &Component) {
            self.output += match component.kind {
                ComponentKind::Component => "participant ",
                ComponentKind::Actor => "actor ",
                ComponentKind::Boundary => "boundary ",
                ComponentKind::Control => "control ",
                ComponentKind::Entity => "entity ",
                ComponentKind::Database => "database ",
                ComponentKind::Collections => "collections ",
                ComponentKind::Queue => "queue ",
            };
            self.emit_component_name(&component.name);
            self.output += " as ";
            self.output += component.identifier;

            if let Some(color) = Self::extract_color(&component.attributes) {
                self.output += " ";
                self.output += color;
            }

            self.output += "\n";
        }

        fn emit_event(&mut self, event: &Event) {
            match event {
                Event::Message(message) => self.emit_message(message),
                Event::Note(note) => self.emit_note(note),
            }
        }

        fn emit_message(&mut self, message: &Message) {
            self.output += message.from;
            self.emit_arrow(&message.arrow, Self::extract_color(&message.attributes));
            self.output += message.to;

            if let Some(message) = message.text.as_ref() {
                self.output += " : ";
                self.emit_message_text(message);
            }

            self.output += "\n";

            if let Some(activation) = message.activation.as_ref() {
                self.output += "activate ";
                self.output += message.to;
                self.output += "\n";

                for event in &activation.events {
                    self.emit_event(event);
                }

                if let Some((attributes, ret_val)) = activation.ret_val.as_ref() {
                    self.output += message.from;
                    self.emit_arrow(
                        &Arrow {
                            left_dot: false,
                            left_head: ArrowHead::Thick,
                            body: ArrowBody::Dash,
                            right_head: ArrowHead::None,
                            right_dot: false,
                        },
                        Self::extract_color(attributes),
                    );
                    self.output += message.to;
                    self.output += " : ";
                    self.output += ret_val;
                    self.output += "\n";
                }
                self.output += "deactivate";

                self.output += "\n";
            }
        }

        fn emit_arrow(&mut self, arrow: &Arrow, color: Option<&str>) {
            if arrow.left_dot {
                self.output += " o";
            } else {
                self.output += " ";
            }

            self.output += match arrow.left_head {
                ArrowHead::None => "",
                ArrowHead::Thick => "<",
                ArrowHead::Thin => "<<",
                ArrowHead::Lost => "x",
            };

            if let Some(color) = color
                && matches!(arrow.direction(), ArrowDirection::Rtl)
            {
                self.output += "[";
                self.output += color;
                self.output += "]";
            }

            self.output += match arrow.body {
                ArrowBody::Solid => "-",
                ArrowBody::Dash => "--",
            };

            if let Some(color) = color
                && (matches!(arrow.direction(), ArrowDirection::Ltr)
                    || matches!(arrow.direction(), ArrowDirection::Bi))
            {
                self.output += "[";
                self.output += color;
                self.output += "]";
            }

            self.output += match arrow.right_head {
                ArrowHead::None => "",
                ArrowHead::Thick => ">",
                ArrowHead::Thin => ">>",
                ArrowHead::Lost => "x",
            };
            if arrow.right_dot {
                self.output += "o ";
            } else {
                self.output += " ";
            }
        }

        fn emit_component_name(&mut self, name: &str) {
            self.output += "\"";
            for c in name.chars() {
                match c {
                    '\\' => self.output += "\\\\",
                    '\n' => self.output += "\\n",
                    c @ '"' => panic!("Unsupported char: {c}"),
                    c => self.output.write_char(c).expect("Can write to string"),
                }
            }
            self.output += "\"";
        }

        fn emit_message_text(&mut self, text: &str) {
            for c in text.chars() {
                match c {
                    '\\' => self.output += "\\\\",
                    '\n' => self.output += "\\n",
                    c => self.output.write_char(c).expect("Can write to string"),
                }
            }
        }

        fn emit_note(&mut self, note: &Note) {
            let pos = note
                .attributes
                .iter()
                .find_map(NotePosition::opt_from)
                .unwrap_or_default();

            self.output += "note ";
            match pos {
                NotePosition::Across => self.output += "across\n",
                NotePosition::Left => self.output += "left\n",
                NotePosition::LeftOf(component) => {
                    self.output += "left of ";
                    self.output += component;
                    self.output += "\n";
                }
                NotePosition::Right => self.output += "right\n",
                NotePosition::RightOf(component) => {
                    self.output += "right of ";
                    self.output += component;
                    self.output += "\n";
                }
            }
            self.output += note.text.as_ref();
            if !note.text.ends_with('\n') {
                self.output += "\n";
            }
            self.output += "end note\n";
        }

        fn extract_color<'a>(attributes: &'a [Attribute]) -> Option<&'a str> {
            // todo error reporting on invalid quoted color and on invalid color name (identifier)
            // todo parse quoted color as either an identifier or an hexcolor
            attributes
                .iter()
                .find(|a| matches!(a.key, Key::Color))
                .map(|a| &a.value)
                .and_then(|v| match v {
                    Value::Identifier(identifier) => crate::colors::COLORS.get(identifier).copied(),
                    Value::Quoted(val) => Some(val.as_ref()),
                    Value::HexColor(color) => Some(*color),
                })
        }
    }

    impl Emitter for &mut PlantUml {
        fn emit_graph(self, graph: &Graph<'_>) {
            self.output += "@startuml\n";
            self.output += "autonumber\n\n";

            for component in &graph.components {
                self.emit_component(component);
            }
            self.output += "\n";

            for event in &graph.events {
                self.emit_event(event);
            }

            self.output += "@enduml\n";
        }
    }

    #[derive(Debug, Default)]
    enum NotePosition<'input> {
        #[default]
        Across,
        Left,
        LeftOf(&'input str),
        Right,
        RightOf(&'input str),
    }

    impl<'input> NotePosition<'input> {
        fn opt_from(attribute: &Attribute<'input>) -> Option<Self> {
            match attribute.key {
                //"left" => Some(Self::Left),
                Key::LeftOf => match attribute.value {
                    Value::Identifier(identifier) => Some(Self::LeftOf(identifier)),
                    Value::Quoted(_) | Value::HexColor(_) => None,
                },
                //"right" => Some(Self::Right),
                Key::RightOf => match attribute.value {
                    Value::Identifier(identifier) => Some(Self::LeftOf(identifier)),
                    Value::Quoted(_) | Value::HexColor(_) => None,
                },
                //Key::Across => Some(Self::Across),
                Key::Color => None,
            }
        }
    }
}
