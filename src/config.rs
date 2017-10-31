use std::ffi::{CStr, CString};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ShareNet {
    Share,
    Unshare,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Config {
    command: CString,
    args: Vec<CString>,
    new_root: Option<CString>,
    share_net: ShareNet,
    redirect_stdin: Option<CString>,
    redirect_stdout: Option<CString>,
    redirect_stderr: Option<CString>,
}

impl Config {
    pub fn new(
        command: CString,
        args: Vec<CString>,
        new_root: Option<CString>,
        share_net: ShareNet,
        redirect_stdin: Option<CString>,
        redirect_stdout: Option<CString>,
        redirect_stderr: Option<CString>,
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

    pub fn command(&self) -> &CStr {
        &self.command
    }

    pub fn args<'a>(&'a self) -> impl Iterator<Item = &'a CStr> {
        self.args.iter().map(|c_string| c_string.as_c_str())
    }

    pub fn new_root(&self) -> Option<&CStr> {
        self.new_root.as_ref().map(|c_string| c_string.as_c_str())
    }

    pub fn share_net(&self) -> ShareNet {
        self.share_net
    }

    pub fn redirect_stdin(&self) -> Option<&CStr> {
        self.redirect_stdin
            .as_ref()
            .map(|c_string| c_string.as_c_str())
    }

    pub fn redirect_stdout(&self) -> Option<&CStr> {
        self.redirect_stdout
            .as_ref()
            .map(|c_string| c_string.as_c_str())
    }

    pub fn redirect_stderr(&self) -> Option<&CStr> {
        self.redirect_stderr
            .as_ref()
            .map(|c_string| c_string.as_c_str())
    }
}
