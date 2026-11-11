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
    use crate::parser::{Arrow, Component, Graph, Message};

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
            self.output += "participant ";
            self.output += &component.name;
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
                self.output += message;
            }

            self.output += "\n";
        }

        fn emit_arrow(&mut self, arrow: &Arrow) {
            self.output += match arrow {
                Arrow::Solid => "->",
                Arrow::Dash => "-->",
            };
        }
    }

    impl Emitter for &mut PlantUml {
        fn emit_graph(self, graph: &Graph<'_>) {
            self.output += "@startuml\n";

            for component in graph.components.iter() {
                self.emit_component(component);
            }

            for message in graph.messages.iter() {
                self.emit_message(message);
            }

            self.output += "@enduml\n";
        }
    }
}
