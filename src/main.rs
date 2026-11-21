#![allow(dead_code)]
#![warn(clippy::pedantic)]

use crate::ast::Graph;
use crate::cli::{Opts, OutputFile};
use crate::emitter::plantuml;
use clap::Parser;
use std::fs;
use std::io::{Read, stdin};
use std::process::exit;

mod ast;
mod checker;
mod cli;
mod colors;
mod emitter;
mod lexer;
mod parser;
mod resolver;

fn main() {
    let opts = Opts::parse();

    let input = match opts.input().map(fs::read_to_string).transpose() {
        Ok(Some(input)) => input,
        Ok(None) => {
            let mut str = String::new();
            match stdin().read_to_string(&mut str) {
                Ok(_) => str,
                Err(e) => {
                    eprintln!("Could not read file: {e}");
                    exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Could not read file: {err}");
            exit(1);
        }
    };

    let graph = Graph::parse(&input)
        .and_then(Graph::check)
        // .inspect(|g| {
        //     dbg!(g);
        // })
        .and_then(|g| g.resolve(&opts.overlays()))
        .unwrap();
    // eprintln!("{graph:#?}");

    let out = plantuml::emit(&graph);

    match opts.output() {
        OutputFile::Stdout => {
            println!("{out}");
        }
        OutputFile::File(file) => {
            if let Err(e) = fs::write(file.as_ref(), out) {
                eprintln!("Could not write file: {e}");
                exit(1);
            }
        }
    }
}
