#![allow(dead_code)]

use crate::parser::parse;

mod ast;
mod colors;
mod emitter;
mod lexer;
mod parser;

fn main() {
    let input = include_str!("../examples/diagram.wip.seq");
    let _ = dbg!(parse(input));
}
