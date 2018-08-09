use std::io::stdin;

fn main() {
    let stdin = stdin();

    // Send A, Receive B
    println!("A");

    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();
    assert_eq!(line, "B\n");
}
