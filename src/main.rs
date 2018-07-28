#![cfg_attr(
    feature = "cargo-clippy",
    deny(
        clone_on_ref_ptr, default_trait_access, doc_markdown, empty_enum,
        empty_line_after_outer_attr, enum_glob_use, expl_impl_clone_on_copy, fallible_impl_from,
        filter_map, float_cmp_const, items_after_statements, match_same_arms,
        multiple_inherent_impl, mut_mut, needless_continue, option_map_unwrap_or,
        option_map_unwrap_or_else, range_plus_one, replace_consts, result_map_unwrap_or_else,
        single_match_else, unimplemented, unnecessary_unwrap, use_self, used_underscore_binding,
        writeln_empty_string, wrong_self_convention
    )
)]
#![deny(
    missing_copy_implementations, missing_debug_implementations, trivial_casts,
    trivial_numeric_casts, unreachable_pub, unused_extern_crates, unused_import_braces,
    unused_qualifications, unused_results, variant_size_differences, warnings
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
    match args::parse().and_then(|(args, output)| Ok((ia_sandbox::run_jail(&args)?, output))) {
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
            let mut fail: &dyn Fail = err.cause();
            eprintln!("{}", fail);
            while let Some(cause) = fail.cause() {
                eprintln!("{}", cause);
                fail = cause;
            }
            process::exit(1);
        }
    }
}
