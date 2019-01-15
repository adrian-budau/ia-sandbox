#![deny(
    clippy::clone_on_ref_ptr,
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::empty_line_after_outer_attr,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::fallible_impl_from,
    clippy::filter_map,
    clippy::float_cmp_const,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    clippy::needless_continue,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::range_plus_one,
    clippy::replace_consts,
    clippy::result_map_unwrap_or_else,
    clippy::single_match_else,
    clippy::unimplemented,
    clippy::unnecessary_unwrap,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::writeln_empty_string,
    clippy::wrong_self_convention,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    warnings
)]

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
    match args::parse()
        .and_then(|(args, output)| Ok((ia_sandbox::spawn_jail(&args)?.wait()?, output)))
    {
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
                        run_info.usage().user_time().as_milliseconds(),
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
            let mut fail: &dyn Fail = err.as_fail();
            eprintln!("{}", fail);
            while let Some(cause) = fail.cause() {
                eprintln!("{}", cause);
                fail = cause;
            }
            process::exit(1);
        }
    }
}
