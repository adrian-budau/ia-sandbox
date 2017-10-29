#![feature(fnbox)]
extern crate eval;

#[macro_use]
extern crate clap;

use eval::config::Config;
use eval::{run_jail, Result};

mod app;
mod args;

use std::process;

macro_rules! eprintln {
    ($($tt:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(&mut ::std::io::stderr(), $($tt)*);
    }}
}

fn main() {
    match args::parse().and_then(run) {
        Ok(()) => process::exit(0),
        Err(err) => {
            eprintln!("{:?}", err);
            process::exit(1);
        }
    }
}

fn run(args: Config) -> Result<()> {
    run_jail(args)
}
