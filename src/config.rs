use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShareNet {
    Share,
    Unshare,
}

/// Limits for memory/time
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Limits {
    wall_time: Option<Duration>,
    user_time: Option<Duration>,
}

impl Limits {
    pub fn new(wall_time: Option<Duration>, user_time: Option<Duration>) -> Limits {
        Limits {
            wall_time,
            user_time,
        }
    }

    pub fn wall_time(&self) -> Option<Duration> {
        self.wall_time
    }

    pub fn user_time(&self) -> Option<Duration> {
        self.user_time
    }
}

impl Default for Limits {
    fn default() -> Limits {
        Limits::new(None, None)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Config {
    command: PathBuf,
    args: Vec<OsString>,
    new_root: Option<PathBuf>,
    share_net: ShareNet,
    redirect_stdin: Option<PathBuf>,
    redirect_stdout: Option<PathBuf>,
    redirect_stderr: Option<PathBuf>,
    limits: Limits,
    instance_name: Option<OsString>,
    cpuacct_controller_path: Option<PathBuf>,
}

impl Config {
    pub fn new(
        command: PathBuf,
        args: Vec<OsString>,
        new_root: Option<PathBuf>,
        share_net: ShareNet,
        redirect_stdin: Option<PathBuf>,
        redirect_stdout: Option<PathBuf>,
        redirect_stderr: Option<PathBuf>,
        limits: Limits,
        instance_name: Option<OsString>,
        cpuacct_controller_path: Option<PathBuf>,
    ) -> Config {
        Config {
            command,
            args,
            new_root,
            share_net,
            redirect_stdin,
            redirect_stdout,
            redirect_stderr,
            limits,
            instance_name,
            cpuacct_controller_path,
        }
    }

    pub fn command(&self) -> &Path {
        &self.command
    }

    pub fn args<'a>(&'a self) -> Vec<&'a OsStr> {
        self.args
            .iter()
            .map(|os_string| os_string.as_os_str())
            .collect()
    }

    pub fn new_root(&self) -> Option<&Path> {
        self.new_root.as_ref().map(|path_buf| path_buf.as_path())
    }

    pub fn share_net(&self) -> ShareNet {
        self.share_net
    }

    pub fn redirect_stdin(&self) -> Option<&Path> {
        self.redirect_stdin
            .as_ref()
            .map(|path_buf| path_buf.as_path())
    }

    pub fn redirect_stdout(&self) -> Option<&Path> {
        self.redirect_stdout
            .as_ref()
            .map(|path_buf| path_buf.as_path())
    }

    pub fn redirect_stderr(&self) -> Option<&Path> {
        self.redirect_stderr
            .as_ref()
            .map(|path_buf| path_buf.as_path())
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    pub fn instance_name(&self) -> Option<&OsStr> {
        self.instance_name
            .as_ref()
            .map(|os_string| os_string.as_os_str())
    }

    pub fn cpuacct_controller_path(&self) -> Option<&Path> {
        self.cpuacct_controller_path
            .as_ref()
            .map(|path_buf| path_buf.as_path())
    }
}
