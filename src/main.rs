#![feature(fnbox)]
#[macro_use]
extern crate clap;
extern crate ia_sandbox;

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
    match args::parse().and_then(ia_sandbox::run_jail) {
        Ok(run_info) => {
            println!("{}", run_info);
            process::exit(0);
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}
