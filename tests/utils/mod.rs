use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

use tempdir;

use ia_sandbox::{self, Result};
use ia_sandbox::config::{Config, Limits, ShareNet};
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
        .filter_map(|token| {
            if token.starts_with("/") {
                Some(token.into())
            } else {
                None
            }
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
    command: PathBuf,
    args: Vec<OsString>,
    new_root: Option<PathBuf>,
    share_net: bool,
    redirect_stdin: Option<PathBuf>,
    redirect_stdout: Option<PathBuf>,
    redirect_stderr: Option<PathBuf>,
    limits: Option<Limits>,
}

impl ConfigBuilder {
    pub fn new<T: AsRef<OsStr>>(command: T) -> ConfigBuilder {
        ConfigBuilder {
            command: command.as_ref().into(),
            args: vec![],
            new_root: None,
            share_net: true,
            redirect_stdin: Some("/dev/null".into()),
            redirect_stdout: Some("/dev/null".into()),
            redirect_stderr: Some("/dev/null".into()),
            limits: None,
        }
    }

    pub fn arg<T: AsRef<OsStr>>(&mut self, arg: T) -> &mut ConfigBuilder {
        self.args.push(arg.as_ref().into());
        self
    }

    pub fn args<I, T>(&mut self, args: I) -> &mut ConfigBuilder
    where
        I: IntoIterator<Item = T>,
        T: AsRef<OsStr>,
    {
        for arg in args.into_iter() {
            self.arg(arg);
        }

        self
    }

    pub fn new_root<T: AsRef<Path>>(&mut self, new_root: T) -> &mut ConfigBuilder {
        self.new_root = Some(new_root.as_ref().into());
        self
    }

    pub fn share_net(&mut self, share_net: bool) -> &mut ConfigBuilder {
        self.share_net = share_net;
        self
    }

    pub fn stdin<T: AsRef<Path>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stdin = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn stdout<T: AsRef<Path>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stdout = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn stderr<T: AsRef<Path>>(&mut self, redirect_stdin: T) -> &mut ConfigBuilder {
        self.redirect_stderr = Some(redirect_stdin.as_ref().into());
        self
    }

    pub fn limits(&mut self, limits: Limits) -> &mut ConfigBuilder {
        self.limits = Some(limits);
        self
    }

    pub fn build_and_run(&mut self) -> Result<RunInfo<()>> {
        let config = Config::new(
            self.command.clone(),
            self.args.clone(),
            self.new_root.clone(),
            if self.share_net {
                ShareNet::Share
            } else {
                ShareNet::Unshare
            },
            self.redirect_stdin.clone(),
            self.redirect_stdout.clone(),
            self.redirect_stderr.clone(),
            self.limits.unwrap_or(Default::default()),
            Some(OsString::from("test")),
            None,
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
