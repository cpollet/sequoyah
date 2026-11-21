#![allow(dead_code)]
#![warn(clippy::pedantic)]

use crate::ast::Graph;
use crate::emitter::plantuml;

mod ast;
mod checker;
mod colors;
mod emitter;
mod lexer;
mod parser;
mod resolver;

fn main() {
    let input = include_str!("../examples/diagram.wip.seq");
    let graph = Graph::parse(input)
        .and_then(Graph::check)
        .and_then(|g| g.resolve(&["*"]))
        .unwrap();
    //eprintln!("{graph:#?}");
    println!("{}", plantuml::emit(&graph));
}
