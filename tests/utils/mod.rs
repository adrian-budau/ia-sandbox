use std::ffi::CString;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ia_sandbox::run_info::RunInfo;

use libc;
use tempfile::{Builder, TempDir};

mod builder;
pub use self::builder::{ConfigBuilder, LimitsBuilder};

pub mod matchers;
use self::matchers::Matcher;

fn get_exec_libs<T>(file: T) -> Vec<PathBuf>
where
    T: AsRef<Path>,
{
    let output = Command::new("ldd")
        .stdin(Stdio::null())
        .arg(file.as_ref())
        .output()
        .unwrap();
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
        }).filter_map(|token| {
            if token.starts_with('/') {
                Some(token.into())
            } else {
                None
            }
        }).collect()
}

// Until https://marc.info/?l=linux-kernel&m=150834137201488 gets resolved, we can't
// use fs::copy on libs/executables
fn copy_by_command<T1: AsRef<Path>, T2: AsRef<Path>>(from: T1, to: T2) {
    assert!(
        Command::new("cp")
            .arg(from.as_ref())
            .arg(to.as_ref())
            .status()
            .expect("failed to copy")
            .success()
    );
}

fn copy_libs<T1, T2>(file: T1, path: T2)
where
    T1: AsRef<Path>,
    T2: AsRef<Path>,
{
    for lib in get_exec_libs(file) {
        let destination = path.as_ref().join(lib.strip_prefix("/").unwrap());
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        copy_by_command(&lib, destination);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PivotRoot {
    Pivot,
    DoNot,
}

pub struct TestRunnerHelper<'a> {
    test_name: &'a str,
    temp_dir: TempDir,
    config_builder: ConfigBuilder,
}

impl<'a> TestRunnerHelper<'a> {
    pub fn for_simple_exec<T: AsRef<Path>>(
        test_name: &str,
        exec_path: T,
        pivot_root: PivotRoot,
    ) -> TestRunnerHelper {
        let temp_dir = Builder::new().prefix(test_name).tempdir().unwrap();
        let exec_path = exec_path.as_ref();

        copy_by_command(
            exec_path,
            temp_dir.path().join(exec_path.file_name().unwrap()),
        );
        if pivot_root == PivotRoot::Pivot {
            copy_libs(exec_path, temp_dir.path());
        }

        let mut config_builder = match pivot_root {
            PivotRoot::Pivot => {
                let mut config_builder =
                    ConfigBuilder::new(Path::new("/").join(exec_path.file_name().unwrap()));
                config_builder.new_root(temp_dir.path());
                config_builder
            }
            PivotRoot::DoNot => ConfigBuilder::new(exec_path),
        };

        config_builder.instance_name(test_name);
        TestRunnerHelper {
            test_name,
            temp_dir,
            config_builder,
        }
    }

    pub fn config_builder(&mut self) -> &mut ConfigBuilder {
        &mut self.config_builder
    }

    pub fn file_path<T: AsRef<Path>>(&mut self, path: T) -> PathBuf {
        self.temp_dir.path().join(path.as_ref())
    }

    pub fn write_file<T: AsRef<Path>>(&mut self, path: T, data: &[u8]) {
        let mut file = File::create(self.file_path(path)).unwrap();
        file.write_all(data).unwrap();
    }

    pub fn read_line<T: AsRef<Path>>(&mut self, path: T) -> String {
        let mut file = File::open(path.as_ref()).unwrap();
        let mut line = String::new();
        file.read_to_string(&mut line).unwrap();
        line
    }
}

impl<'a> Drop for TestRunnerHelper<'a> {
    fn drop(&mut self) {
        // Clear the cgroups folders
        // we might get here because of a panic, so make sure not to panic again
        fs::remove_dir(Path::new("/sys/fs/cgroup/cpuacct/ia-sandbox").join(self.test_name))
            .unwrap_or(());
        fs::remove_dir(Path::new("/sys/fs/cgroup/memory/ia-sandbox").join(self.test_name))
            .unwrap_or(());
        fs::remove_dir(Path::new("/sys/fs/cgroup/pids/ia-sandbox").join(self.test_name))
            .unwrap_or(());
    }
}

pub fn make_fifo(path: &Path) {
    let path_c_string = CString::new(path.as_os_str().as_bytes()).unwrap();

    if unsafe { libc::mkfifo(path_c_string.as_ptr(), 0o666) } == -1 {
        panic!("Could not make fifo at {}", path.display());
    }
}

pub trait RunInfoExt {
    fn assert<F: Matcher>(self, matcher: F);
}

impl RunInfoExt for RunInfo<()> {
    fn assert<F: Matcher>(self, matcher: F) {
        match matcher.try_match(self) {
            Ok(()) => (),
            Err(err) => panic!("assertion failed: {}\n{}", matcher.assertion_string(), err),
        }
    }
}
