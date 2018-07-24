use std::ffi::OsString;
use std::ops;
use std::path::PathBuf;
use std::result;
use std::time::Duration;

use ia_sandbox::config::{
    Config, ControllerPath, Limits, Mount, MountOptions, ShareNet, SpaceUsage,
};

use app;
use clap;
use failure::{self, ResultExt};

type Result<T> = result::Result<T, failure::Error>;

pub fn parse() -> Result<(Config, OutputType)> {
    let matches = app::app().get_matches();
    ArgMatches(matches).to_config_and_output()
}

pub enum OutputType {
    Human,
    Oneline,
    Json,
}

struct ArgMatches<'a>(clap::ArgMatches<'a>);

impl<'a> ops::Deref for ArgMatches<'a> {
    type Target = clap::ArgMatches<'a>;
    fn deref(&self) -> &clap::ArgMatches<'a> {
        &self.0
    }
}

fn parse_duration(string: &str) -> Result<Duration> {
    let number_index = string
        .find(|c: char| !c.is_digit(10))
        .ok_or_else(|| format_err!("Could not find duration suffix (s/ns/ms): {}", string))?;
    let (number, suffix) = string.split_at(number_index);
    let number = number
        .parse::<u64>()
        .context(format_err!("Could not parse number {}", number))?;
    match suffix {
        "ns" => Ok(Duration::from_nanos(number)),
        "ms" => Ok(Duration::from_millis(number)),
        "s" => Ok(Duration::from_secs(number)),
        suffix => Err(format_err!("Unrecognized suffix: {}", suffix)),
    }
}

fn parse_space_usage(string: &str) -> Result<SpaceUsage> {
    let number_index = string.find(|c: char| !c.is_digit(10)).ok_or_else(|| {
        format_err!(
            "Could not find duration suffix (b/kb/mb/gb/kib/mib/gib): {}",
            string
        )
    })?;

    let (number, suffix) = string.split_at(number_index);
    let number = number
        .parse::<u64>()
        .context(format_err!("Could not parse number {}", number))?;
    match suffix {
        "b" => Ok(SpaceUsage::from_bytes(number)),
        "kb" => Ok(SpaceUsage::from_kilobytes(number)),
        "mb" => Ok(SpaceUsage::from_megabytes(number)),
        "gb" => Ok(SpaceUsage::from_gigabytes(number)),
        "kib" => Ok(SpaceUsage::from_kibibytes(number)),
        "mib" => Ok(SpaceUsage::from_mebibytes(number)),
        "gib" => Ok(SpaceUsage::from_gibibytes(number)),
        suffix => Err(format_err!("Unrecognized suffix: {}", suffix)),
    }
}

fn parse_mount_options(string: &str) -> Result<MountOptions> {
    let mut mount_options = MountOptions::default();

    for option in string.split(',') {
        match option {
            "rw" => mount_options.set_read_only(false),
            "dev" => mount_options.set_dev(true),
            "exec" => mount_options.set_exec(true),
            _ => {
                return Err(format_err!(
                    "Could not parse mount option, unrecognized `{}`",
                    option
                ))
            }
        }
    }
    Ok(mount_options)
}

fn parse_mount(string: &str) -> Result<Mount> {
    let parts: Vec<&str> = string.split(':').collect();

    match *parts.as_slice() {
        [source] => Ok(Mount::new(
            PathBuf::from(source),
            PathBuf::from(source),
            MountOptions::default(),
        )),
        [source, destination] => Ok(Mount::new(
            PathBuf::from(source),
            PathBuf::from(destination),
            MountOptions::default(),
        )),
        [source, destination, options] => Ok(Mount::new(
            PathBuf::from(source),
            PathBuf::from(destination),
            parse_mount_options(options)?,
        )),
        _ => Err(format_err!("Could not parse mount")),
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
    fn to_config_and_output(&self) -> Result<(Config, OutputType)> {
        let limits = Limits::new(
            self.wall_time()?,
            self.user_time()?,
            self.memory()?,
            self.stack()?,
            self.pids()?,
        );
        let controller_path = ControllerPath::new(
            self.cpuacct_controller_path(),
            self.memory_controller_path(),
            self.pids_controller_path(),
        );

        let config = Config::new(
            self.command()?,
            self.args(),
            self.new_root(),
            self.share_net(),
            self.redirect_stdin(),
            self.redirect_stdout(),
            self.redirect_stderr(),
            limits,
            self.instance_name(),
            controller_path,
            self.mounts()?,
        );

        Ok((config, self.output_type()))
    }

    fn command(&self) -> Result<PathBuf> {
        self.value_of_os("COMMAND")
            .ok_or_else(|| format_err!("No command was specified"))
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
        if self.is_present("share-net") {
            ShareNet::Share
        } else {
            ShareNet::Unshare
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

    fn wall_time(&self) -> Result<Option<Duration>> {
        Ok(
            flip_option_result(self.value_of("wall-time").map(|x| parse_duration(x)))
                .context("Could not parse wall time")?,
        )
    }

    fn user_time(&self) -> Result<Option<Duration>> {
        Ok(
            flip_option_result(self.value_of("time").map(|x| parse_duration(x)))
                .context("Could not parse time")?,
        )
    }

    fn memory(&self) -> Result<Option<SpaceUsage>> {
        Ok(
            flip_option_result(self.value_of("memory").map(|x| parse_space_usage(x)))
                .context("Could not parse memory")?,
        )
    }

    fn stack(&self) -> Result<Option<SpaceUsage>> {
        Ok(
            flip_option_result(self.value_of("stack").map(|x| parse_space_usage(x)))
                .context("Could not parse stack")?,
        )
    }

    fn pids(&self) -> Result<Option<usize>> {
        flip_option_result(
            self.value_of("pids")
                .map(|x| Ok(x.parse::<usize>().context("Could not parse pids")?)),
        )
    }

    fn instance_name(&self) -> Option<OsString> {
        self.value_of_os("instance-name")
            .map(|os_str| os_str.to_os_string())
    }

    fn cpuacct_controller_path(&self) -> Option<PathBuf> {
        self.value_of_os("cpuacct-path").map(PathBuf::from)
    }

    fn memory_controller_path(&self) -> Option<PathBuf> {
        self.value_of_os("memory-path").map(PathBuf::from)
    }

    fn pids_controller_path(&self) -> Option<PathBuf> {
        self.value_of_os("pids-path").map(PathBuf::from)
    }

    fn output_type(&self) -> OutputType {
        match self.value_of("output").expect("output value") {
            "human" => OutputType::Human,
            "oneline" => OutputType::Oneline,
            "json" => OutputType::Json,
            _ => unreachable!(),
        }
    }

    fn mounts(&self) -> Result<Vec<Mount>> {
        match self.values_of("mount") {
            None => Ok(vec![]),
            Some(args) => args.map(parse_mount).collect(),
        }
    }
}
