use std::env;
use std::process;

fn main() {
    process::exit(env::args().last().unwrap().trim().parse().unwrap());
}
