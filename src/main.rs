#![feature(fnbox)]
#![feature(duration_extras)]
#![feature(slice_patterns)]
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate ia_sandbox;
extern crate serde_json;

use failure::Fail;
use std::process;

use std::io;

use ia_sandbox::utils::DurationExt;

mod app;
mod args;
use args::OutputType;

fn main() {
    match args::parse().and_then(|(args, output)| Ok((ia_sandbox::run_jail(args)?, output))) {
        Ok((run_info, output)) => {
            match output {
                OutputType::Human => println!("{}", run_info),
                OutputType::Oneline => {
                    if run_info.is_success() {
                        print!("OK: ");
                    } else {
                        print!("FAIL: ");
                    }
                    println!(
                        "time {}ms memory {}kb: {}",
                        run_info.usage().user_time().as_millis(),
                        run_info.usage().memory().as_kilobytes(),
                        run_info.result()
                    );
                }
                OutputType::Json => {
                    let stdout = io::stdout();
                    serde_json::to_writer_pretty(stdout.lock(), &run_info).unwrap();
                }
            };
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
