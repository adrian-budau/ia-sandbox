use std::boxed::FnBox;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::iter;
use std::marker::PhantomData;
use std::os::unix::io::FromRawFd;
use std::path::Path;
use std::ptr;
use std::result::Result as StdResult;

use bincode;
use libc::{self, CLONE_NEWIPC, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUSER, CLONE_NEWUTS,
           CLONE_VFORK, SIGCHLD};
use serde::Serialize;
use serde::de::DeserializeOwned;

use config::ShareNet;
use errors::{ChildError, ChildResult, Result, ResultExt};
use run_info::{RunInfo, RunInfoResult};

const DEFAULT_STACK_SIZE: usize = 256 * 1024;
const CLONE_NEWNET: libc::c_int = 0x40000000;


#[derive(Debug, Eq, PartialEq)]
pub struct UserId(libc::uid_t);
#[derive(Debug, Eq, PartialEq)]
pub struct GroupId(libc::gid_t);

pub fn get_user_group_id() -> (UserId, GroupId) {
    return unsafe { (UserId(libc::getuid()), GroupId(libc::getgid())) };
}

pub fn set_uid_gid_maps((uid, gid): (UserId, GroupId)) -> ChildResult<()> {
    let uid_error = |_| ChildError::WriteUidError(last_error_string());
    let mut uid_map = OpenOptions::new()
        .write(true)
        .open("/proc/self/uid_map")
        .map_err(&uid_error)?;

    uid_map
        .write_all(format!("0 {} 1\n", uid.0).as_bytes())
        .map_err(&uid_error)?;

    // We need to set /proc/self/setgroups to deny for writing the gid_map to succeed
    let mut setgroups = OpenOptions::new()
        .write(true)
        .open("/proc/self/setgroups")
        .map_err(|_| {
            ChildError::WriteGidError(format!(
                "Could not open setgroups file: {}",
                last_error_string()
            ))
        })?;
    setgroups.write_all(b"deny").map_err(|_| {
        ChildError::WriteGidError(format!(
            "Could not write \"deny\" to setgroups file: {}",
            last_error_string()
        ))
    })?;

    let gid_error = |_| ChildError::WriteGidError(last_error_string());
    let mut gid_map = OpenOptions::new()
        .write(true)
        .open("/proc/self/gid_map")
        .map_err(&gid_error)?;
    gid_map
        .write_all(format!("0 {} 1\n", gid.0).as_bytes())
        .map_err(&gid_error)?;

    Ok(())
}

pub fn clone<F, T: Debug>(f: F, share_net: ShareNet) -> Result<CloneHandle<T>>
where
    F: FnOnce() -> T + Send,
    T: Serialize,
{
    let mut clone_flags = CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWIPC | CLONE_NEWUTS | CLONE_NEWNS
        | CLONE_VFORK | SIGCHLD;
    if share_net == ShareNet::Unshare {
        clone_flags |= CLONE_NEWNET;
    }

    let mut child_stack = vec![0; DEFAULT_STACK_SIZE];

    extern "C" fn cb(arg: *mut libc::c_void) -> libc::c_int {
        unsafe { Box::from_raw(arg as *mut Box<FnBox()>)() };


        return 0;
    }

    let (read_error_pipe, mut write_error_pipe) = make_pipe()?;
    let f: Box<FnBox() + Send> = Box::new(move || {
        let result = f();
        let _ = bincode::serialize_into(&mut write_error_pipe, &result, bincode::Infinite);
    });
    let f = Box::new(f);

    let pid = match unsafe {
        #[allow(trivial_casts)]
        libc::clone(
            cb,
            child_stack.as_mut_ptr().offset(child_stack.len() as isize) as *mut libc::c_void,
            clone_flags,
            &*f as *const _ as *mut libc::c_void,
        )
    } {
        -1 => return Err(format!("Clone Error: {}", last_error_string()).into()),
        x => x,
    };

    Ok(CloneHandle {
        pid,
        read_error_pipe,
        phantom: PhantomData,
    })
}

const OLD_ROOT_NAME: &'static str = ".old_root";
pub fn pivot_root<F>(new_root: &CStr, before_umount: F) -> ChildResult<()>
where
    F: FnOnce() -> ChildResult<()>,
{
    let old_root = format!("{}/{}", new_root.to_string_lossy(), OLD_ROOT_NAME);

    // create an .old_root folder in case it doesn't exist
    if !Path::new(&old_root).exists() {
        fs::create_dir(&old_root)
            .map_err(|err| ChildError::CreateDirError(err.description().into()))?;
    }

    let old_root = CString::new(old_root).unwrap();
    // bind mount it on top of itself (this is necessary for pivot_root to work)
    // it must also be a private mount (and everything under it as well)
    let res = unsafe {
        libc::mount(
            new_root.as_ptr(),
            new_root.as_ptr(),
            ptr::null_mut(),
            libc::MS_REC | libc::MS_BIND | libc::MS_PRIVATE,
            ptr::null_mut(),
        )
    };

    if res == -1 {
        return Err(ChildError::MountError {
            path: new_root.to_string_lossy().into_owned(),
            error: last_error_string(),
        });
    }

    // Change directory first
    if unsafe { libc::chdir(new_root.as_ptr()) } == -1 {
        return Err(ChildError::ChdirError(last_error_string()));
    }

    sys_pivot_root(new_root, &old_root)?;

    // Change root (not needed in practice because the implementation of pivot_root
    // does change the root, though it's not guaranteed in the future)
    let root = CString::new(".").unwrap();
    if unsafe { libc::chroot(root.as_ptr()) } == -1 {
        return Err(ChildError::ChrootError(last_error_string()));
    }

    before_umount()?;

    // And unmount .old_root
    let old_root = CString::new(format!("/{}", OLD_ROOT_NAME)).unwrap();
    if unsafe { libc::umount2(old_root.as_ptr(), libc::MNT_DETACH) } == -1 {
        Err(ChildError::MountError {
            path: old_root.to_string_lossy().into_owned(),
            error: format!("Unmounting error: {}", last_error_string()),
        })
    } else {
        Ok(())
    }
}

pub fn mount_proc() -> ChildResult<()> {
    let name = CString::new("proc").unwrap();
    let path = "/proc";

    // create /proc in case it doesn't exist (likely first time we pivot_root)
    if !Path::new(&path).exists() {
        fs::create_dir(&path).map_err(|err| ChildError::CreateDirError(err.description().into()))?;
    }
    let path = CString::new(path).unwrap();

    let res = unsafe {
        libc::mount(
            name.as_ptr(),
            path.as_ptr(),
            name.as_ptr(),
            0,
            ptr::null_mut(),
        )
    };

    if res == -1 {
        Err(ChildError::MountError {
            path: "/proc".into(),
            error: last_error_string(),
        })
    } else {
        Ok(())
    }
}

const EXEC_RETRIES: usize = 3;
const RETRY_DELAY: libc::c_uint = 100000;
pub fn exec_command<'a, 'b: 'a, T>(path: &'b CStr, arguments: T) -> ChildResult<()>
where
    T: IntoIterator<Item = &'a CStr>,
{
    let arguments: Vec<_> = iter::once(path)
        .chain(arguments)
        .map(|string| string.as_ptr()) // convert to C pointers
        .chain(iter::once(ptr::null())) // add an ending NULL
        .collect();

    for retry in 0..EXEC_RETRIES {
        let res = unsafe { libc::execv(path.as_ptr(), arguments.as_slice().as_ptr()) };

        if res == -1 {
            let error = errno::Errno::last_error();
            if error.error_code() != libc::ETXTBSY || retry == EXEC_RETRIES - 1 {
                return Err(ChildError::ExecError(error.error_string()));
            }
            let res = unsafe { libc::usleep(RETRY_DELAY) };
            if res == -1 {
                return Err(ChildError::UsleepError(last_error_string()));
            }
        } else {
            return Ok(());
        }
    }
    unreachable!()
}

pub struct Fd(libc::c_int, &'static str, libc::c_int, libc::c_int);

pub const STDIN: Fd = Fd(0, "stdin", libc::O_RDONLY, 0);
pub const STDOUT: Fd = Fd(
    1,
    "stdout",
    libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
    0o666,
);
pub const STDERR: Fd = Fd(
    2,
    "stderr",
    libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
    0o666,
);

pub fn redirect_fd(fd: Fd, path: &CStr) -> ChildResult<()> {
    if unsafe { libc::close(fd.0) } == -1 {
        return Err(ChildError::CloseFdError {
            fd: fd.0,
            name: fd.1.into(),
            error: last_error_string(),
        });
    }

    match unsafe { libc::open(path.as_ptr(), fd.2, fd.3) } {
        -1 => Err(ChildError::OpenFdError {
            fd: fd.0,
            name: fd.1.into(),
            error: last_error_string(),
        }),
        x if x != fd.0 => Err(ChildError::OpenFdError {
            fd: fd.0,
            name: fd.1.into(),
            error: format!("Wrong file descritor opened {}", x),
        }),
        _ => Ok(()),
    }
}

fn sys_pivot_root(new_root: &CStr, old_root: &CStr) -> ChildResult<()> {
    match unsafe { libc::syscall(libc::SYS_pivot_root, new_root.as_ptr(), old_root.as_ptr()) } {
        -1 => Err(ChildError::PivotRootError({
            let errno = errno::Errno::last_error();
            match errno.error_code() {
                libc::EBUSY => "new_root or put_old are on the current root\
                                filesystem, or a filesystem is already\
                                mounted on put_old."
                    .into(),
                libc::EINVAL => "put_old is not underneath new_root".into(),
                libc::ENOTDIR => "new_root or put_old is not a directory".into(),
                libc::EPERM => "The calling process does not have the\
                                CAP_SYS_ADMIN capability"
                    .into(),
                _ => errno.error_string(),
            }
        })),
        _ => Ok(()),
    }
}

mod errno {
    use libc;
    use std::ffi::CStr;

    #[derive(Debug)]
    pub struct Errno(libc::c_int);

    impl Errno {
        pub fn last_error() -> Errno {
            unsafe { Errno(*libc::__errno_location()) }
        }

        pub fn error_code(&self) -> libc::c_int {
            self.0
        }

        pub fn error_string(&self) -> String {
            let buffer = &mut [0i8; 256];
            if unsafe { libc::strerror_r(self.0, buffer.as_mut_ptr(), buffer.len()) } == -1 {
                return "unexpected strerror_r error".into();
            }
            unsafe {
                CStr::from_ptr(buffer.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }
}

fn last_error_string() -> String {
    errno::Errno::last_error().error_string()
}

fn make_pipe() -> Result<(File, File)> {
    unsafe {
        let fd = &mut [0; 2];
        if libc::pipe2(fd.as_mut_ptr(), libc::O_CLOEXEC) == -1 {
            Err("Error on pipe".into())
        } else {
            Ok((File::from_raw_fd(fd[0]), File::from_raw_fd(fd[1])))
        }
    }
}

pub struct CloneHandle<T> {
    pid: libc::pid_t,
    read_error_pipe: File,
    phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> CloneHandle<T> {
    pub fn wait(mut self) -> Result<StdResult<RunInfo, T>> {
        let mut data = Vec::new();
        let _ = self.read_error_pipe
            .read_to_end(&mut data)
            .chain_err(|| "Error reading from error pipe")?;
        if data.len() > 0 {
            return bincode::deserialize(&data)
                .map(|x| Err(x))
                .chain_err(|| "Bincode decode problem");
        }

        loop {
            let mut status: libc::c_int = 0;
            match unsafe { libc::waitpid(self.pid, &mut status, 0) } {
                -1 => return Err(format!("Error with waitpid").into()),
                _ => {
                    if unsafe { libc::WIFEXITED(status) } {
                        let exit_code = unsafe { libc::WEXITSTATUS(status) };
                        if exit_code == 0 {
                            return Ok(Ok(RunInfo::new(RunInfoResult::Success)));
                        } else {
                            return Ok(Ok(
                                RunInfo::new(RunInfoResult::NonZeroExitStatus(exit_code)),
                            ));
                        }
                    }

                    if unsafe { libc::WIFSIGNALED(status) } {
                        let signal = unsafe { libc::WTERMSIG(status) };
                        return Ok(Ok(RunInfo::new(RunInfoResult::KilledBySignal(signal))));
                    }

                    if unsafe { libc::WIFSTOPPED(status) || libc::WIFCONTINUED(status) } {
                        return Err("Child process stopped/continued unexpected".into());
                    }
                }
            }
        }
    }
}
