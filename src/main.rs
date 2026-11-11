mod emitter;
mod parser;

fn main() {
    let input = include_str!("../examples/diagram.seq");
    let graph = parser::parse(input).unwrap();
    println!("{}", emitter::plantuml::emit(&graph));
}
