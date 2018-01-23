#![feature(fnbox)]
#![feature(duration_extras)]
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate ia_sandbox;

use failure::Fail;
use std::process;

mod app;
mod args;

macro_rules! eprintln {
    ($($tt:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(&mut ::std::io::stderr(), $($tt)*);
    }}
}

fn main() {
    match args::parse().and_then(|args| Ok(ia_sandbox::run_jail(args)?)) {
        Ok(run_info) => {
            println!("{}", run_info);
            process::exit(0);
        }
        Err(err) => {
            let mut fail: &Fail = err.cause();
            eprintln!("{}", fail);
            while let Some(cause) = fail.cause() {
                eprintln!("{}", cause);
                fail = cause;
            }
            process::exit(1);
        }
    }
}
