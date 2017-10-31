use std::io::stdin;
use std::process;

fn main() {
    let stdin = stdin();
    let mut line = String::new();
    stdin.read_line(&mut line).unwrap();
    println!("{}", line);
    process::exit(line.trim().parse().unwrap());
}
