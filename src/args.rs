use std::ffi::OsString;
use std::ops;
use std::path::PathBuf;
use std::result;

use ia_sandbox::config::{Config, ShareNet};

use app;
use clap;
use failure;

type Result<T> = result::Result<T, failure::Error>;

pub fn parse() -> Result<Config> {
    let matches = app::app().get_matches();
    ArgMatches(matches).to_config()
}

struct ArgMatches<'a>(clap::ArgMatches<'a>);

impl<'a> ops::Deref for ArgMatches<'a> {
    type Target = clap::ArgMatches<'a>;
    fn deref(&self) -> &clap::ArgMatches<'a> {
        &self.0
    }
}

impl<'a> ArgMatches<'a> {
    fn to_config(&self) -> Result<Config> {
        Ok(Config::new(
            self.command()?,
            self.args(),
            self.new_root(),
            self.share_net(),
            self.redirect_stdin(),
            self.redirect_stdout(),
            self.redirect_stderr(),
        ))
    }

    fn command(&self) -> Result<PathBuf> {
        self.value_of_os("COMMAND")
            .ok_or(format_err!("No command was specified"))
            .map(PathBuf::from)
    }

    fn args(&self) -> Vec<OsString> {
        match self.values_of_os("ARGS") {
            None => vec![],
            Some(vals) => vals.map(|x| x.to_os_string()).collect(),
        }
    }

    fn new_root(&self) -> Option<PathBuf> {
        self.value_of_os("new-root").map(PathBuf::from)
    }

    fn share_net(&self) -> ShareNet {
        match self.is_present("share-net") {
            true => ShareNet::Share,
            false => ShareNet::Unshare,
        }
    }

    fn redirect_stdin(&self) -> Option<PathBuf> {
        self.value_of_os("stdin").map(PathBuf::from)
    }

    fn redirect_stdout(&self) -> Option<PathBuf> {
        self.value_of_os("stdout").map(PathBuf::from)
    }

    fn redirect_stderr(&self) -> Option<PathBuf> {
        self.value_of_os("stderr").map(PathBuf::from)
    }
}
