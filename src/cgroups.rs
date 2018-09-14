use std::cmp;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::result;
use std::str::FromStr;
use std::time::Duration;

use config::{ClearUsage, ControllerPath, Limits, SpaceUsage};
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

const ISOLATED_CGROUP_NAME: &str = "isolated";
pub(crate) fn enter_cgroup(controller_path: &Path) -> Result<()> {
    let isolated_cgroup = controller_path.join(ISOLATED_CGROUP_NAME);

    if !isolated_cgroup.exists() {
        fs::create_dir(&isolated_cgroup).map_err(|err| {
            CGroupError::InstanceControllerCreateError {
                controller_path: controller_path.to_path_buf(),
                instance_name: OsString::from(ISOLATED_CGROUP_NAME),
                error: err.description().into(),
            }
        })?;
    }

    cgroup_write(&isolated_cgroup, "tasks", format!("{}\n", ffi::getpid()))
}

const DEFAULT_INSTANCE_NAME: &str = "default";
fn get_instance_path(controller_path: &Path, instance_name: Option<&OsStr>) -> Result<PathBuf> {
    let instance = instance_name.unwrap_or_else(|| OsStr::new(DEFAULT_INSTANCE_NAME));
    if !controller_path.exists() {
        return Err(CGroupError::ControllerMissing(
            controller_path.to_path_buf(),
        ));
    }

    let instance_path = controller_path.join(instance);
    if !instance_path.exists() {
        fs::create_dir(&instance_path).map_err(|err| {
            CGroupError::InstanceControllerCreateError {
                controller_path: controller_path.to_path_buf(),
                instance_name: instance.to_os_string(),
                error: err.description().into(),
            }
        })?;
    }
    Ok(instance_path)
}

const CPUACCT_DEFAULT_CONTROLLER_PATH: &str = "/sys/fs/cgroup/cpuacct/ia-sandbox";
pub(crate) fn enter_cpuacct_cgroup(
    controller_path: Option<&Path>,
    instance_name: Option<&OsStr>,
    clear_usage: ClearUsage,
) -> Result<()> {
    let instance_path = get_instance_path(
        controller_path.unwrap_or_else(|| Path::new(CPUACCT_DEFAULT_CONTROLLER_PATH)),
        instance_name,
    )?;

    if clear_usage == ClearUsage::Yes {
        cgroup_write(&instance_path, "cpuacct.usage", "0\n")?;
    }

    enter_cgroup(&instance_path)
}

const MEMORY_DEFAULT_CONTROLLER_PATH: &str = "/sys/fs/cgroup/memory/ia-sandbox";
const EXTRA_MEMORY_GIVEN: u64 = 16 * 1_024;
pub(crate) fn enter_memory_cgroup(
    controller_path: Option<&Path>,
    instance_name: Option<&OsStr>,
    memory_limit: Option<SpaceUsage>,
    clear_usage: ClearUsage,
) -> Result<()> {
    let instance_path = get_instance_path(
        controller_path.unwrap_or_else(|| Path::new(MEMORY_DEFAULT_CONTROLLER_PATH)),
        instance_name,
    )?;

    if clear_usage == ClearUsage::Yes {
        cgroup_write(&instance_path, "memory.max_usage_in_bytes", "0\n")?;
        cgroup_write(&instance_path, "memory.memsw.max_usage_in_bytes", "0\n").unwrap_or(());

        // Reset limits to infinite in case there is no memory limit but also because we need at all
        // times for limit_in_bytes < memsw.limit_in_bytes
        cgroup_write(&instance_path, "memory.memsw.limit_in_bytes", "-1\n").unwrap_or(());
        cgroup_write(&instance_path, "memory.limit_in_bytes", "-1\n")?;
        if let Some(memory_limit) = memory_limit {
            // Assign some extra memory so that we can tell when a killed by signal 9 is actually a
            // memory limit exceeded
            cgroup_write(
                &instance_path,
                "memory.limit_in_bytes",
                format!("{}\n", memory_limit.as_bytes() + EXTRA_MEMORY_GIVEN),
            )?;
            cgroup_write(
                &instance_path,
                "memory.memsw.limit_in_bytes",
                format!("{}\n", memory_limit.as_bytes() + EXTRA_MEMORY_GIVEN),
            ).unwrap_or(());
        }
    }

    enter_cgroup(&instance_path)
}

const PIDS_DEFAULT_CONTROLLER_PATH: &str = "/sys/fs/cgroup/pids/ia-sandbox";
pub(crate) fn enter_pids_cgroup(
    controller_path: Option<&Path>,
    instance_name: Option<&OsStr>,
    pids_limit: Option<usize>,
    clear_usage: ClearUsage,
) -> Result<()> {
    let instance_path = get_instance_path(
        controller_path.unwrap_or_else(|| Path::new(PIDS_DEFAULT_CONTROLLER_PATH)),
        instance_name,
    )?;

    if clear_usage == ClearUsage::Yes {
        if let Some(pids_limit) = pids_limit {
            cgroup_write(&instance_path, "pids.max", format!("{}\n", pids_limit))?;
        } else {
            cgroup_write(&instance_path, "pids.max", "max\n")?;
        }
    }

    enter_cgroup(&instance_path)
}

pub(crate) fn enter_all_cgroups(
    controller_path: &ControllerPath,
    instance_name: Option<&OsStr>,
    limits: Limits,
    clear_usage: ClearUsage,
) -> Result<()> {
    enter_cpuacct_cgroup(controller_path.cpuacct(), instance_name, clear_usage)?;
    enter_memory_cgroup(
        controller_path.memory(),
        instance_name,
        limits.memory(),
        clear_usage,
    )?;
    enter_pids_cgroup(
        controller_path.pids(),
        instance_name,
        limits.pids(),
        clear_usage,
    )
}

pub(crate) fn get_usage(
    controller_path: &ControllerPath,
    instance_name: Option<&OsStr>,
    wall_time: Duration,
) -> Result<RunUsage> {
    let cpuacct_controller_path = controller_path
        .cpuacct()
        .unwrap_or_else(|| Path::new(CPUACCT_DEFAULT_CONTROLLER_PATH));
    let memory_controller_path = controller_path
        .memory()
        .unwrap_or_else(|| Path::new(MEMORY_DEFAULT_CONTROLLER_PATH));
    let instance = instance_name.unwrap_or_else(|| OsStr::new(DEFAULT_INSTANCE_NAME));

    let cpuacct_instance_path = cpuacct_controller_path.join(instance);
    let user_time = Duration::from_nanos(cgroup_read(&cpuacct_instance_path, "cpuacct.usage")?);

    let memory_instance_path = memory_controller_path.join(instance);
    let memory = SpaceUsage::from_bytes(cmp::max(
        cgroup_read(&memory_instance_path, "memory.max_usage_in_bytes")?,
        cgroup_read(&memory_instance_path, "memory.memsw.max_usage_in_bytes").unwrap_or(0),
    ));
    Ok(RunUsage::new(user_time, wall_time, memory))
}
