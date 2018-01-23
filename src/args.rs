use std::ffi::OsString;
use std::ops;
use std::path::PathBuf;
use std::result;

use ia_sandbox::config::{Config, Limits, ShareNet};

use app;
use clap;
use failure::{self, ResultExt};

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

fn parse_duration(string: &str) -> Result<u64> {
    let number_index = string
        .find(|c: char| !c.is_digit(10))
        .ok_or(format_err!("Could not find duration suffix (s/ns/ms): {}", string))?;
    let (number, suffix) = string.split_at(number_index);
    let number = number.parse::<u64>().context(format_err!("Could not parse number {}", number))?;
    match suffix {
        "ns" => Ok(number),
        "ms" => Ok(number * 1_000_000),
        "s" => Ok(number * 1_000_000_000),
        suffix => Err(format_err!("Unrecognized suffix: {}", suffix).into()),
    }
}

fn flip_option_result<T>(arg: Option<Result<T>>) -> Result<Option<T>> {
    match arg {
        None => Ok(None),
        Some(Ok(x)) => Ok(Some(x)),
        Some(Err(err)) => Err(err),
    }
}

impl<'a> ArgMatches<'a> {
    fn to_config(&self) -> Result<Config> {
        let limits = Limits::new(self.wall_time()?);
        Ok(Config::new(
            self.command()?,
            self.args(),
            self.new_root(),
            self.share_net(),
            self.redirect_stdin(),
            self.redirect_stdout(),
            self.redirect_stderr(),
            limits,
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

    fn wall_time(&self) -> Result<Option<u64>> {
        Ok(flip_option_result(self.value_of("wall-time").map(|x| parse_duration(x))).context("Could not parse wall time")?)
    }
}
