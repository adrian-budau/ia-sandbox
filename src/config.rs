use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShareNet {
    Share,
    Unshare,
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
    ) -> Config {
        Config {
            command,
            args,
            new_root,
            share_net,
            redirect_stdin,
            redirect_stdout,
            redirect_stderr,
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
}
