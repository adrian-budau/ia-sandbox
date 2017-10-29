#![feature(fnbox)]
extern crate ia_sandbox;

#[macro_use]
extern crate clap;

use ia_sandbox::config::Config;
use ia_sandbox::{run_jail, Result};

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
