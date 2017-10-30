#![feature(fnbox)]
#[macro_use]
extern crate clap;
extern crate ia_sandbox;

use std::process;

use ia_sandbox::{run_jail, Result};
use ia_sandbox::config::Config;

mod app;
mod args;


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
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn run(args: Config) -> Result<()> {
    run_jail(args)
}
