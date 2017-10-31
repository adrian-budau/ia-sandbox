use std::ffi::CString;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

use tempdir;

use ia_sandbox::{self, Result};
use ia_sandbox::config::{Config, ShareNet};
use ia_sandbox::run_info::RunInfo;

fn get_exec_libs<T>(file: T) -> Vec<PathBuf>
where
    T: AsRef<Path>,
{
    let output = Command::new("ldd").arg(file.as_ref()).output().unwrap();

    if !output.status.success() {
        return vec![];
    }

    BufReader::new(output.stdout.as_slice())
        .lines()
        .flat_map(|line| {
            line.unwrap()
                .split_whitespace()
                .map(|s| s.to_owned())
                .collect::<Vec<_>>()
        })
        .filter_map(|token| if token.starts_with("/") {
            Some(token.into())
        } else {
            None
        })
        .collect()
}

fn copy_libs<T1, T2>(file: T1, path: T2)
where
    T1: AsRef<Path>,
    T2: AsRef<Path>,
{
    for lib in get_exec_libs(file) {
        let destination = path.as_ref().join(lib.strip_prefix("/").unwrap());
        for parent in destination.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::copy(&lib, destination).unwrap();
    }
}

pub fn with_setup<'a, 'b, I, T1, T2, F>(prefix: &'b str, files: I, cb: F)
where
    I: IntoIterator<Item = &'a (T1, T2)>,
    T1: AsRef<Path> + 'a,
    T2: AsRef<Path> + 'a,
    F: for<'c> FnOnce(&'c Path),
{
    let temp_dir = tempdir::TempDir::new(prefix).unwrap();

    for &(ref source, ref target) in files {
        fs::copy(
            &source,
            temp_dir
                .path()
                .join(target.as_ref().strip_prefix("/").unwrap()),
        ).unwrap();
        copy_libs(source, temp_dir.path());
    }
    cb(temp_dir.path());
}

pub struct ConfigBuilder {
    command: String,
    args: Vec<String>,
    new_root: Option<String>,
    share_net: bool,
    redirect_stdin: Option<String>,
    redirect_stdout: Option<String>,
    redirect_stderr: Option<String>,
}

impl ConfigBuilder {
    pub fn new<T: AsRef<str>>(command: T) -> ConfigBuilder {
        ConfigBuilder {
            command: command.as_ref().into(),
            args: vec![],
            new_root: None,
            share_net: true,
            redirect_stdin: Some("/dev/null".into()),
            redirect_stdout: Some("/dev/null".into()),
            redirect_stderr: Some("/dev/null".into()),
        }
    }

    pub fn arg<T: AsRef<str>>(&mut self, arg: T) -> &mut ConfigBuilder {
        self.args.push(arg.as_ref().into());
        self
    }

    pub fn args<I, T>(&mut self, args: I) -> &mut ConfigBuilder
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        for arg in args.into_iter() {
            self.arg(arg);
        }

        self
    }

    pub fn new_root<T: AsRef<str>>(&mut self, new_root: T) -> &mut ConfigBuilder {
        self.new_root = Some(new_root.as_ref().into());
        self
    }

    pub fn share_net(&mut self, share_net: bool) -> &mut ConfigBuilder {
        self.share_net = share_net;
        self
    }

    pub fn stdin<T: AsRef<str>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stdin = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn stdout<T: AsRef<str>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stdout = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn stderr<T: AsRef<str>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stderr = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn build_and_run(&mut self) -> Result<RunInfo> {
        fn to_cstring<T: AsRef<str>>(string: T) -> CString {
            CString::new(string.as_ref()).unwrap()
        }

        let config = Config::new(
            to_cstring(&self.command),
            self.args.iter().map(to_cstring).collect(),
            self.new_root.as_ref().map(to_cstring),
            if self.share_net {
                ShareNet::Share
            } else {
                ShareNet::Unshare
            },
            self.redirect_stdin.as_ref().map(to_cstring),
            self.redirect_stdout.as_ref().map(to_cstring),
            self.redirect_stderr.as_ref().map(to_cstring),
        );

        ia_sandbox::run_jail(config)
    }
}

macro_rules! matches {
    ($e:expr, $p: pat) => {
        match $e {
            $p => true,
            _ => false
        }
    }
}
