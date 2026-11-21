#![allow(dead_code)]
#![warn(clippy::pedantic)]

use crate::parser::parse;

mod ast;
mod colors;
mod emitter;
mod lexer;
mod parser;

fn main() {
    let input = include_str!("../examples/diagram.wip.seq");
    let graph = parse(input).unwrap();
    eprintln!("{graph:#?}");
    println!("{}", emitter::plantuml::emit(&graph));
}
