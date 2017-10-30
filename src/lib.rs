#![recursion_limit = "1024"]
#![feature(fnbox)]
#![feature(conservative_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![deny(missing_copy_implementations, missing_debug_implementations,
        trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results, variant_size_differences, warnings)]

extern crate bincode;
#[macro_use]
extern crate error_chain;
extern crate libc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod meta;
pub mod errors;
pub mod config;
mod ffi;

use config::Config;
pub use errors::*;


pub fn run_jail(config: Config) -> Result<()> {
    let user_group_id = ffi::get_user_group_id();
    let handle = ffi::clone(
        || {
            if let Some(new_root) = config.new_root() {
                ffi::pivot_root(new_root, || {
                    // Mount proc (since we are in a new pid namespace)
                    // Must be done after pivot_root so we mount this in the right location
                    // but also before we unmount the old root because ... I don't know
                    ffi::mount_proc()
                })?;
            } else {
                ffi::mount_proc()?;
            }

            // Make sure we are root (we don't really need to,
            // but this way the child process can do anything it likes
            // inside its namespace and nothing outside)
            // Must be done after mount_proc so we can properly read and write
            // /proc/self/uid_map and /proc/self/gid_map
            ffi::set_uid_gid_maps(user_group_id)?;

            ffi::exec_command(config.command(), config.args())
        },
        config.share_net(),
    )?;
    handle
        .wait()
        .and_then(|child_error: Option<ChildResult<()>>| {
            child_error.unwrap_or(Ok(())).map_err(|e| e.into())
        })
}
