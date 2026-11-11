mod emitter;
mod parser;

fn main() {
    let input = include_str!("../examples/diagram.seq");
    let graph = parser::parse(input).unwrap();
    //dbg!(&graph);
    println!("{}", emitter::plantuml::emit(&graph));
}
