extern crate ia_sandbox;
extern crate tempdir;

use std::ffi::CString;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

use ia_sandbox::config::{Config, ShareNet};

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

fn with_setup<'a, 'b, I, T1, T2, F>(prefix: &'b str, files: I, cb: F)
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

const HELLO_WORLD: [(&'static str, &'static str); 1] =
    [("./target/debug/examples/hello_world", "/hello_world")];

#[test]
fn test_basic_sandbox() {
    with_setup("test_basic_sandbox", HELLO_WORLD.iter(), |dir| {
        let config = Config::new(
            CString::new(dir.join("hello_world").to_string_lossy().into_owned()).unwrap(),
            vec![],
            None,
            ShareNet::Share,
        );

        ia_sandbox::run_jail(config).unwrap();
    });
}

#[test]
fn test_exec_failed() {
    with_setup("test_exec_failed", HELLO_WORLD[..].iter(), |dir| {
        let config = Config::new(
            CString::new(dir.join("missing").to_string_lossy().into_owned()).unwrap(),
            vec![],
            None,
            ShareNet::Share,
        );

        let result = ia_sandbox::run_jail(config);
        match result {
            Err(
                ia_sandbox::errors::Error(
                    ia_sandbox::errors::ErrorKind::ChildError(
                        ia_sandbox::errors::ChildError::ExecError(_),
                    ),
                    _,
                ),
            ) => {
                return;
            }
            Ok(_) => assert!(false),
            _ => (),
        }

        result.unwrap();
    });
}

#[test]
fn test_pivot_root() {
    with_setup("test_pivot_root", HELLO_WORLD[..].iter(), |dir| {
        let config = Config::new(
            CString::new("/hello_world").unwrap(),
            vec![],
            Some(CString::new(dir.to_string_lossy().into_owned()).unwrap()),
            ShareNet::Share,
        );

        ia_sandbox::run_jail(config).unwrap();
    });
}
