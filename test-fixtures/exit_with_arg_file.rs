use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn main() {
    let path = env::args().last().unwrap();

    let mut file = BufReader::new(File::open(path).unwrap());
    let mut line = String::new();
    file.read_line(&mut line).unwrap();
    process::exit(line.trim().parse().unwrap());
}
