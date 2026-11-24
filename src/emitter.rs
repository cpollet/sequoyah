use crate::ir::Graph;

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
    use crate::emitter::{Emit, Emitter};
    use crate::ir::{
        Arrow, ArrowBody, ArrowDirection, ArrowHead, Component, ComponentKind, Event, Graph,
        Message, Note, Position, Shape,
    };
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

            if let Some(color) = component.color {
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
            self.emit_arrow(&message.arrow, message.color);
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

                if let Some(ret_val) = activation.ret_val.as_ref() {
                    self.output += message.from;
                    self.emit_arrow(
                        &Arrow {
                            left_dot: false,
                            left_head: ArrowHead::Thick,
                            body: ArrowBody::Dash,
                            right_head: ArrowHead::None,
                            right_dot: false,
                        },
                        ret_val.color,
                    );
                    self.output += message.to;
                    self.output += " : ";
                    self.output += ret_val.text.as_ref();
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
            // todo implement across with a list of identifiers
            // todo implement aligned notes (/)
            // todo implement formatting (markdown?)

            match note.shape {
                None | Some(Shape::Default) => self.output += "note ",
                Some(Shape::Rectangle) => self.output += "rnote ",
                Some(Shape::Hexagonal) => self.output += "hnote ",
            }

            match note.position {
                None | Some(Position::Across) => self.output += "across",
                Some(Position::Left) => self.output += "left",
                Some(Position::LeftOf(component)) => {
                    self.output += "left of ";
                    self.output += component;
                }
                Some(Position::Right) => self.output += "right",
                Some(Position::RightOf(component)) => {
                    self.output += "right of ";
                    self.output += component;
                }
            }
            if let Some(color) = note.color {
                self.output += color;
            }
            self.output += "\n";
            self.output += note.text.as_ref();
            if !note.text.ends_with('\n') {
                self.output += "\n";
            }
            self.output += "end note\n";
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
}
