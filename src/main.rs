mod parser;

fn main() {
    let input = include_str!("../examples/diagram.seq");
    println!("{:#?}", parser::parse(input).unwrap());
}
