mod parser;

fn main() {
    // let input = include_str!("../examples/diagram.seq");
    // println!("{:?}", parser::parse(input).unwrap());

    println!(
        "{:?}",
        parser::parse_component("component \"browser\" as b;").unwrap()
    );

    // A message with the optional `: text` part.
    println!("{:?}", parser::parse_message("b -> s : GET /path").unwrap());
    // A message without the optional `: text` part.
    println!("{:?}", parser::parse_message("s -> d").unwrap());
    // A newline in the text must be written as the `\n` escape.
    println!(
        "{:?}",
        parser::parse_message("b -> s : first line\\nsecond line").unwrap()
    );
}
