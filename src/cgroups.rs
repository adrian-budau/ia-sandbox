use std::error::Error;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::result;
use std::str::FromStr;
use std::time::Duration;

use errors::CGroupError;
use ffi;
use run_info::RunUsage;

type Result<T> = result::Result<T, CGroupError>;

fn cgroup_write<T1: AsRef<Path>, T2: AsRef<str>>(
    controller_path: &Path,
    file: T1,
    line: T2,
) -> Result<()> {
    let path = controller_path.join(file.as_ref());
    let mut cgroup_file = OpenOptions::new().write(true).open(&path).map_err(|err| {
        CGroupError::OpenCGroupFileError {
            controller_path: controller_path.to_path_buf(),
            file: file.as_ref().to_path_buf(),
            error: err.description().into(),
        }
    })?;

    cgroup_file
        .write_all(line.as_ref().as_bytes())
        .map_err(|err| CGroupError::WriteCGroupFileError {
            controller_path: controller_path.to_path_buf(),
            file: file.as_ref().to_path_buf(),
            error: err.description().into(),
        })
}

fn cgroup_read<T1: AsRef<Path>, T2: FromStr>(controller_path: &Path, file: T1) -> Result<T2>
where
    <T2 as FromStr>::Err: Error,
{
    let path = controller_path.join(file.as_ref());
    let mut cgroup_file = OpenOptions::new().read(true).open(&path).map_err(|err| {
        CGroupError::OpenCGroupFileError {
            controller_path: controller_path.to_path_buf(),
            file: file.as_ref().to_path_buf(),
            error: err.description().into(),
        }
    })?;

    let mut buffer = String::new();
    let _ = cgroup_file.read_to_string(&mut buffer).map_err(|err| {
        CGroupError::ReadCGroupFileError {
            controller_path: controller_path.to_path_buf(),
            file: file.as_ref().to_path_buf(),
            error: err.description().into(),
        }
    })?;

    buffer
        .trim()
        .parse::<T2>()
        .map_err(|err| CGroupError::ParseCGroupFileError {
            controller_path: controller_path.to_path_buf(),
            file: file.as_ref().to_path_buf(),
            buffer,
            error: err.description().into(),
        })
}

pub fn enter_cgroup(controller_path: &Path) -> Result<()> {
    cgroup_write(controller_path, "tasks", format!("{}\n", ffi::getpid()))
}

const CPUACCT_DEFAULT_CONTROLLER_PATH: &'static str = "/sys/fs/cgroup/cpuacct/ia-sandbox";
const DEFAULT_INSTANCE_NAME: &'static str = "default";

pub fn enter_cpuacct_cgroup(
    controller_path: Option<&Path>,
    instance_name: Option<&OsStr>,
) -> Result<()> {
    let path = controller_path.unwrap_or(Path::new(CPUACCT_DEFAULT_CONTROLLER_PATH));
    let instance = instance_name.unwrap_or(OsStr::new(DEFAULT_INSTANCE_NAME));
    if !path.exists() {
        return Err(CGroupError::ControllerMissing(path.to_path_buf()));
    }

    let instance_path = path.join(instance);
    if !instance_path.exists() {
        fs::create_dir(&instance_path).map_err(|err| CGroupError::InstanceControllerCreateError {
            controller_path: path.to_path_buf(),
            instance_name: instance.to_os_string(),
            error: err.description().into(),
        })?;
    }

    enter_cgroup(&instance_path)?;
    cgroup_write(&instance_path, "cpuacct.usage", "0\n")
}

pub fn get_usage(
    controller_path: Option<&Path>,
    instance_name: Option<&OsStr>,
) -> Result<RunUsage> {
    let path = controller_path.unwrap_or(Path::new(CPUACCT_DEFAULT_CONTROLLER_PATH));
    let instance = instance_name.unwrap_or(OsStr::new(DEFAULT_INSTANCE_NAME));

    let instance_path = path.join(instance);
    Ok(RunUsage::new(Duration::from_nanos(cgroup_read(
        &instance_path,
        "cpuacct.usage",
    )?)))
}
