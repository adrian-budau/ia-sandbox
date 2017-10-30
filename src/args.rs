use std::ffi::{CString, OsStr};
use std::ops;

use ia_sandbox::{Result, ResultExt};
use ia_sandbox::config::{Config, ShareNet};

use app;
use clap;

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

fn to_cstring(string: &OsStr) -> Result<CString> {
    string
        .to_str()
        .ok_or_else(|| "Could not parse OsStr as String".into())
        .and_then(|s| {
            CString::new(s).chain_err(|| "Could not convert arg to CString")
        })
}

impl<'a> ArgMatches<'a> {
    fn to_config(&self) -> Result<Config> {
        Ok(Config::new(
            self.command()?,
            self.args()?,
            self.new_root(),
            self.share_net(),
        ))
    }

    fn command(&self) -> Result<CString> {
        self.value_of_os("COMMAND")
            .ok_or("No command was specified".into())
            .and_then(|x| to_cstring(&x))
    }

    fn args(&self) -> Result<Vec<CString>> {
        match self.values_of_os("ARGS") {
            None => Ok(vec![]),
            Some(vals) => vals.map(|x| to_cstring(&x.to_os_string())).collect(),
        }
    }

    fn new_root(&self) -> Option<CString> {
        self.value_of_os("new-root")
            .and_then(|x| to_cstring(x).ok())
    }

    fn share_net(&self) -> ShareNet {
        match self.is_present("share-net") {
            true => ShareNet::Share,
            false => ShareNet::Unshare,
        }
    }
}
