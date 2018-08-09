use std::io::stdin;

fn main() {
    let stdin = stdin();

    // Receive A, Send B
    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();
    assert_eq!(line, "A\n");

    println!("B");
}
