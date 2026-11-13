use crate::parser::Graph;

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
    use crate::parser::{Arrow, ArrowBody, ArrowHead, Component, ComponentKind, Graph, Message};
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
            self.output += component.alias;
            self.output += "\n";
        }

        fn emit_message(&mut self, message: &Message) {
            self.output += message.from;
            self.emit_arrow(&message.arrow);
            self.output += message.to;

            if let Some(message) = message.message.as_ref() {
                self.output += " : ";
                self.emit_message_text(message);
            }

            self.output += "\n";

            if let Some(activation) = message.activation.as_ref() {
                self.output += "activate ";
                self.output += message.to;
                self.output += "\n";

                for message in activation.messages.iter() {
                    self.emit_message(message);
                }

                if let Some(ret_val) = activation.ret_val.as_ref() {
                    self.output += message.from;
                    self.emit_arrow(&Arrow {
                        left_dot: false,
                        left_head: ArrowHead::Thick,
                        body: ArrowBody::Dash,
                        right_head: ArrowHead::None,
                        right_dot: false,
                    });
                    self.output += message.to;
                    self.output += " : ";
                    self.output += ret_val;
                    self.output += "\n";
                }
                self.output += "deactivate";

                self.output += "\n";
            }
        }

        fn emit_arrow(&mut self, arrow: &Arrow) {
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
            self.output += match arrow.body {
                ArrowBody::Solid => "-",
                ArrowBody::Dash => "--",
            };
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
    }

    impl Emitter for &mut PlantUml {
        fn emit_graph(self, graph: &Graph<'_>) {
            self.output += "@startuml\n";

            for component in graph.components.iter() {
                self.emit_component(component);
            }
            self.output += "\n";

            for message in graph.messages.iter() {
                self.emit_message(message);
            }

            self.output += "@enduml\n";
        }
    }
}
