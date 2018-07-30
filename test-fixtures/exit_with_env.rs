use std::env;
use std::process;

fn main() {
    let arg = env::var("arg").unwrap();

    process::exit(arg.trim().parse().unwrap());
}
