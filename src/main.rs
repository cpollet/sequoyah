mod parser;

fn main() {
    // let input = include_str!("../examples/diagram.seq");
    // println!("{:?}", parser::parse(input).unwrap());

    println!(
        "{:?}",
        parser::parse_component("component \"browser\" as b;").unwrap()
    );

    println!("{:?}", parser::parse_message("b -> s : GET /path").unwrap());
    println!(
        "{:?}",
        parser::parse_message("b -> s [color=red, ] : GET /path").unwrap()
    );
    println!("{:?}", parser::parse_message("s -> d").unwrap());
    println!(
        "{:?}",
        parser::parse_message("b -> s : first line\\nsecond line").unwrap()
    );
}
