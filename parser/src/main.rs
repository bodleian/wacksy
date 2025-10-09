use std::fs::read;

fn main() {
    let file = read("parser/parsing_test.txt").unwrap();
    println!("{file:?}");
}
