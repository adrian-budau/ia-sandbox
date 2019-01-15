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
    clippy::print_stdout,
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

extern crate bincode;
#[macro_use]
extern crate failure;
extern crate libc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod cgroups;
pub mod config;
pub mod errors;
mod ffi;
pub mod run_info;
pub mod utils;

use config::{Config, Interactive, Limits, ShareNet, SwapRedirects};
pub use errors::*;
use ffi::CloneHandle;
use run_info::{RunInfo, RunUsage};

pub fn spawn_jail(config: &Config) -> Result<JailHandle> {
    let user_group_id = ffi::get_user_group_id();

    ffi::set_sig_alarm_handler().map_err(Error::FFIError)?;

    // Start a supervisor process in a different pid namespace
    // If by any chance the supervisor process dies, by rules of pid namespaces
    // all its descendant processes will die as well
    ffi::clone(ShareNet::Share, false, || {
        ffi::kill_on_parent_death()?;
        // Mount proc just for security
        ffi::mount_proc()?;
        // Without setting uid/gid maps user is not seen so it can not do anything
        ffi::set_uid_gid_maps(user_group_id)?;

        ffi::clone(config.share_net(), true, || {
            if config.swap_redirects() == SwapRedirects::Yes {
                if let Some(stdout) = config.redirect_stdout() {
                    ffi::redirect_fd(ffi::STDOUT, stdout)?;
                }
            }

            if let Some(stdin) = config.redirect_stdin() {
                ffi::redirect_fd(ffi::STDIN, stdin)?;
            }

            if config.swap_redirects() == SwapRedirects::No {
                if let Some(stdout) = config.redirect_stdout() {
                    ffi::redirect_fd(ffi::STDOUT, stdout)?;
                }
            }

            if let Some(stderr) = config.redirect_stderr() {
                ffi::redirect_fd(ffi::STDERR, stderr)?;
            }

            ffi::set_stack_limit(config.limits().stack())?;
            // Enter cgroup before we pivot root, then it is too late
            cgroups::enter_all_cgroups(
                config.controller_path(),
                config.instance_name(),
                config.limits(),
                config.clear_usage(),
            )?;

            ffi::unshare_cgroup()?;

            // Remount everything privately
            ffi::remount_private()?;

            if let Some(new_root) = config.new_root() {
                for mount in config.mounts() {
                    ffi::mount_inside(new_root, mount)?;
                }

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

            if config.interactive() == Interactive::No {
                // Move the process to a different process group (so it can't kill it's own
                // father by sending signals to the whole process group)
                // But for interactive applications (mostly to test stuff), leave it there
                ffi::move_to_different_process_group()?;
            }

            ffi::exec_command(config.command(), &config.args(), config.environment())?;

            Ok(())
        })?
        .wait(config.limits(), |wall_time| {
            Ok(cgroups::get_usage(
                config.controller_path(),
                config.instance_name(),
                wall_time,
            )?)
        })
        .and_then(|run_info| {
            run_info.and_then(|option| match option {
                None => Ok(()),
                Some(result) => result.map_err(Error::ChildError),
            })
        })
    })
    .map(JailHandle::new)
    .map_err(Error::from)
}

#[allow(missing_debug_implementations)]
pub struct JailHandle {
    handle: CloneHandle<Result<RunInfo<()>>>,
}

impl JailHandle {
    fn new(handle: CloneHandle<Result<RunInfo<()>>>) -> Self {
        Self { handle }
    }

    pub fn wait(self) -> Result<RunInfo<()>> {
        self.handle
            .wait(Limits::default(), |_| Ok(RunUsage::default()))
            .and_then(|run_info| {
                run_info
                    .success() // we only care if supervisor process successfully finished
                    .and_then(|x| x) // its an option inside an option, so flatten it
                    .ok_or(Error::SupervisorProcessDiedError)
                    .and_then(|x| x) // result in result, flatten it
            })
    }
}
