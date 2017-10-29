#![feature(fnbox)]
#![feature(conservative_impl_trait)]

extern crate bincode;
#[macro_use]
extern crate error_chain;
extern crate libc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod errors {
    error_chain!{}
}

pub mod config;
mod ffi;

use config::Config;

pub use errors::*;


pub fn run_jail(config: Config) -> Result<()> {
    let user_group_id = ffi::get_user_group_id();
    let handle = ffi::clone(
        || {
            for new_root in config.new_root() {
                ffi::pivot_root(new_root)?;
            }

            // Mount proc (since we are in a new pid namespace)
            // Must be done after pivot_root so we mount this in the right location
            ffi::mount_proc()?;

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
        .and_then(|child_error: Option<ffi::ChildResult<()>>| {
            child_error.unwrap_or(Ok(())).chain_err(|| "Child error")
        })
}
