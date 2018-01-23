#![recursion_limit = "1024"]
#![feature(fnbox)]
#![feature(conservative_impl_trait)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![deny(missing_copy_implementations, missing_debug_implementations, trivial_casts,
        trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
        unused_results, variant_size_differences, warnings)]

extern crate bincode;
#[macro_use]
extern crate failure;
extern crate libc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod errors;
pub mod run_info;
pub mod config;
mod ffi;

use config::{Config, ShareNet};
pub use errors::*;
use run_info::RunInfo;

pub fn run_jail(config: Config) -> Result<RunInfo<()>> {
    let user_group_id = ffi::get_user_group_id();

    ffi::set_sig_alarm_handler().map_err(Error::FFIError)?;

    // Start a supervisor process in a different pid namespace
    // If by any chance the supervisor process dies, by rules of pid namespaces
    // all its descendant processes will die as well
    ffi::clone(ShareNet::Share, || {
        ffi::kill_on_parent_death().map_err(Error::FFIError)?;
        // Mount proc just for security
        ffi::mount_proc().map_err(Error::FFIError)?;
        // Without setting uid/gid maps user is not seen so it can not do anything
        ffi::set_uid_gid_maps(user_group_id).map_err(Error::FFIError)?;

        ffi::clone(config.share_net(), || {
            if let Some(stdin) = config.redirect_stdin() {
                ffi::redirect_fd(ffi::STDIN, stdin)?;
            }

            if let Some(stdout) = config.redirect_stdout() {
                ffi::redirect_fd(ffi::STDOUT, stdout)?;
            }

            if let Some(stderr) = config.redirect_stderr() {
                ffi::redirect_fd(ffi::STDERR, stderr)?;
            }

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
            ffi::set_uid_gid_maps((ffi::UserId::ROOT, ffi::GroupId::ROOT))?;

            // Move the process to a different process group (so it can't kill it's own
            // father by sending signals to the whole process group)
            ffi::move_to_different_process_group()?;

            ffi::exec_command(config.command(), &config.args())
        }).map_err(Error::FFIError)?
            .wait(config.limits().wall_time())
            .and_then(|run_info| {
                run_info.and_then(|option| match option {
                    None => Ok(()),
                    Some(result) => result.map_err(Error::ChildError),
                })
            })
    }).map_err(Error::FFIError)?
        .wait(None)
        .and_then(|run_info| {
            run_info
                .success() // we only care if supervisor process successfully finished
                .and_then(|x| x) // its an option inside an option, so flatten it
                .ok_or(Error::SupervisorProcessDiedError)
                .and_then(|x| x) // result in result, flatten it
        })
}
