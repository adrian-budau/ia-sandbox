use std::boxed::FnBox;
use std::error::Error as ErrorExt;
use std::ffi::{CString, OsStr};
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::path::{Path, PathBuf};
use std::ptr;
use std::result::Result as StdResult;
use std::time::Instant;

use bincode;
use libc::{self, CLONE_NEWIPC, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUSER, CLONE_NEWUTS,
           CLONE_VFORK, SIGCHLD};
use serde::Serialize;
use serde::de::DeserializeOwned;

use config::{Limits, ShareNet};
use errors::{Error, FFIError};
use run_info::{RunInfo, RunInfoResult, RunUsage};

type Result<T> = StdResult<T, FFIError>;

const DEFAULT_STACK_SIZE: usize = 256 * 1024;
const CLONE_NEWNET: libc::c_int = 0x40000000;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct UserId(libc::uid_t);

impl UserId {
    pub const ROOT: UserId = UserId(0);
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct GroupId(libc::gid_t);

impl GroupId {
    pub const ROOT: GroupId = GroupId(0);
}

pub fn get_user_group_id() -> (UserId, GroupId) {
    unsafe { (UserId(libc::getuid()), GroupId(libc::getgid())) }
}

pub fn getpid() -> libc::c_int {
    unsafe { libc::getpid() }
}
pub fn set_uid_gid_maps((uid, gid): (UserId, GroupId)) -> Result<()> {
    let uid_error = |_| FFIError::WriteUidError(last_error_string());
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
        .map_err(|_| FFIError::WriteSetGroupsError(last_error_string()))?;
    setgroups
        .write_all(b"deny")
        .map_err(|_| FFIError::WriteSetGroupsError(last_error_string()))?;

    let gid_error = |_| FFIError::WriteGidError(last_error_string());
    let mut gid_map = OpenOptions::new()
        .write(true)
        .open("/proc/self/gid_map")
        .map_err(&gid_error)?;
    gid_map
        .write_all(format!("0 {} 1\n", gid.0).as_bytes())
        .map_err(&gid_error)?;

    Ok(())
}

#[allow(trivial_casts)]
pub fn set_sig_alarm_handler() -> Result<()> {
    extern "C" fn handler(_: libc::c_int, _: *mut libc::siginfo_t, _: *mut libc::c_void) {}

    let mut sigaction: libc::sigaction = unsafe { mem::uninitialized() };
    sigaction.sa_flags = libc::SA_SIGINFO;
    sigaction.sa_sigaction =
        unsafe { mem::transmute::<_, libc::sighandler_t>(handler as extern "C" fn(_, _, _)) };
    let _ = unsafe { libc::sigemptyset(&mut sigaction.sa_mask) };
    if unsafe { libc::sigaction(libc::SIGALRM, &mut sigaction, ptr::null_mut()) } == -1 {
        Err(FFIError::SigActionError {
            signal: "SIGALRM".into(),
            error: last_error_string(),
        })
    } else {
        Ok(())
    }
}

pub fn set_alarm_interval(interval: i64) -> Result<()> {
    let timeval = libc::timeval {
        tv_sec: interval / 1_000_000,
        tv_usec: interval % 1_000_000,
    };

    let itimerval = libc::itimerval {
        it_interval: timeval,
        it_value: timeval,
    };

    if unsafe {
        #[allow(trivial_casts)]
        libc::syscall(
            libc::SYS_setitimer,
            libc::ITIMER_REAL,
            &itimerval as *const libc::itimerval,
            ptr::null() as *const libc::itimerval,
        )
    } == -1
    {
        Err(FFIError::SetITimerError(last_error_string()))
    } else {
        Ok(())
    }
}

/// how often SIGALRM should trigger (i microseconds)
const ALARM_TIMER_INTERVAL: i64 = 1_000;

pub fn clone<F, T: Debug>(share_net: ShareNet, f: F) -> Result<CloneHandle<T>>
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
        -1 => return Err(FFIError::CloneError(last_error_string())),
        x => x,
    };

    set_alarm_interval(ALARM_TIMER_INTERVAL)?;

    Ok(CloneHandle {
        pid,
        read_error_pipe,
        phantom: PhantomData,
    })
}

const OLD_ROOT_NAME: &'static str = ".old_root";
pub fn pivot_root<F>(new_root: &Path, before_umount: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let old_root = new_root.join(OLD_ROOT_NAME);

    // create an .old_root folder in case it doesn't exist
    if !old_root.exists() {
        fs::create_dir(&old_root).map_err(|error| FFIError::CreateDirError {
            path: old_root.to_path_buf(),
            error: error.description().into(),
        })?;
    }

    let new_root_c_string = os_str_to_c_string(&new_root);
    // bind mount it on top of itself (this is necessary for pivot_root to work)
    // it must also be a private mount (and everything under it as well)
    let res = unsafe {
        libc::mount(
            new_root_c_string.as_ptr(),
            new_root_c_string.as_ptr(),
            ptr::null_mut(),
            libc::MS_REC | libc::MS_BIND | libc::MS_PRIVATE,
            ptr::null_mut(),
        )
    };

    if res == -1 {
        return Err(FFIError::MountError {
            path: new_root.to_path_buf(),
            error: last_error_string(),
        });
    }

    // Change directory first
    if unsafe { libc::chdir(new_root_c_string.as_ptr()) } == -1 {
        return Err(FFIError::ChdirError {
            path: new_root.to_path_buf(),
            error: last_error_string(),
        });
    }

    sys_pivot_root(new_root, &old_root)?;

    // Change root (not needed in practice because the implementation of pivot_root
    // does change the root, though it's not guaranteed in the future)
    let root = CString::new(".").unwrap();
    if unsafe { libc::chroot(root.as_ptr()) } == -1 {
        return Err(FFIError::ChrootError {
            path: ".".into(),
            error: last_error_string(),
        });
    }

    before_umount()?;

    // And unmount .old_root
    let old_root = Path::new("/").join(OLD_ROOT_NAME);
    let old_root_c_string = os_str_to_c_string(&old_root);
    if unsafe { libc::umount2(old_root_c_string.as_ptr(), libc::MNT_DETACH) } == -1 {
        Err(FFIError::UMountError {
            path: old_root,
            error: last_error_string(),
        })
    } else {
        Ok(())
    }
}

pub fn mount_proc() -> Result<()> {
    let name = CString::new("proc").unwrap();
    let path = PathBuf::from("/proc");

    // create /proc in case it doesn't exist (likely first time we pivot_root)
    if !path.exists() {
        fs::create_dir(&path).map_err(|err| FFIError::CreateDirError {
            path: path.clone(),
            error: err.description().into(),
        })?;
    }
    let path_as_c_string = os_str_to_c_string(&path);

    let res = unsafe {
        libc::mount(
            name.as_ptr(),
            path_as_c_string.as_ptr(),
            name.as_ptr(),
            0,
            ptr::null_mut(),
        )
    };

    if res == -1 {
        Err(FFIError::MountError {
            path: path,
            error: last_error_string(),
        })
    } else {
        Ok(())
    }
}

const EXEC_RETRIES: usize = 3;
const RETRY_DELAY: libc::c_uint = 100000;
pub fn exec_command(command: &Path, arguments: &[&OsStr]) -> Result<()> {
    let arguments_c_string: Vec<_> = iter::once(os_str_to_c_string(command))
        .chain(arguments.iter().map(os_str_to_c_string)) // convert to C pointers
        .collect();

    let arguments_with_null_ending: Vec<_> = arguments_c_string.iter()
        .map(|c_string| c_string.as_ptr())
        .chain(iter::once(ptr::null())) // add an ending NULL
        .collect();
    let command_as_c_string = os_str_to_c_string(command);
    for retry in 0..EXEC_RETRIES {
        let res = unsafe {
            libc::execv(
                command_as_c_string.as_ptr(),
                arguments_with_null_ending.as_slice().as_ptr(),
            )
        };

        if res == -1 {
            let error = errno::Errno::last_error();
            if error.error_code() != libc::ETXTBSY || retry == EXEC_RETRIES - 1 {
                return Err(FFIError::ExecError {
                    command: command.to_path_buf(),
                    arguments: arguments.iter().map(|&os_str| os_str.to_owned()).collect(),
                    error: error.error_string(),
                });
            }
            let res = unsafe { libc::usleep(RETRY_DELAY) };
            if res == -1 {
                return Err(FFIError::UsleepError {
                    time: RETRY_DELAY,
                    error: last_error_string(),
                });
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

pub fn redirect_fd(fd: Fd, path: &Path) -> Result<()> {
    if unsafe { libc::close(fd.0) } == -1 {
        return Err(FFIError::CloseFdError {
            fd: fd.0,
            name: fd.1.into(),
            error: last_error_string(),
        });
    }

    let path_as_c_string = os_str_to_c_string(path);
    match unsafe { libc::open(path_as_c_string.as_ptr(), fd.2, fd.3) } {
        -1 => Err(FFIError::OpenFdError {
            fd: fd.0,
            name: fd.1.into(),
            error: last_error_string(),
        }),
        x if x != fd.0 => Err(FFIError::OpenFdError {
            fd: fd.0,
            name: fd.1.into(),
            error: format!("Wrong file descritor opened {}", x),
        }),
        _ => Ok(()),
    }
}

pub fn move_to_different_process_group() -> Result<()> {
    if unsafe { libc::setpgid(0, 0) } == -1 {
        Err(FFIError::SetpgidError {
            pid: 0,
            pgid: 0,
            error: last_error_string(),
        })
    } else {
        Ok(())
    }
}

fn sys_pivot_root(new_root: &Path, old_root: &Path) -> Result<()> {
    let new_root_c_string = os_str_to_c_string(new_root);
    let old_root_c_string = os_str_to_c_string(old_root);
    match unsafe {
        libc::syscall(
            libc::SYS_pivot_root,
            new_root_c_string.as_ptr(),
            old_root_c_string.as_ptr(),
        )
    } {
        -1 => {
            let errno = errno::Errno::last_error();
            let error = match errno.error_code() {
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
            };

            Err(FFIError::PivotRootError {
                new_root: new_root.to_path_buf(),
                old_root: old_root.to_path_buf(),
                error,
            })
        }
        _ => Ok(()),
    }
}

pub fn kill_on_parent_death() -> Result<()> {
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } == -1 {
        Err(FFIError::PrSetPDeathSigError(last_error_string()))
    } else {
        Ok(())
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
            Err(FFIError::Pipe2Error(last_error_string()))
        } else {
            Ok((File::from_raw_fd(fd[0]), File::from_raw_fd(fd[1])))
        }
    }
}

// This should not fail on linux, considering valid os strings do not contain null
fn os_str_to_c_string<T: AsRef<OsStr>>(os_str: T) -> CString {
    CString::new(os_str.as_ref().as_bytes()).unwrap()
}

pub struct CloneHandle<T> {
    pid: libc::pid_t,
    read_error_pipe: File,
    phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> CloneHandle<T> {
    pub fn wait<F: Fn() -> StdResult<RunUsage, Error>>(
        mut self,
        limits: Limits,
        usage: F,
    ) -> StdResult<RunInfo<Option<T>>, Error> {
        let timer = Instant::now();
        let mut data = Vec::new();
        let _ = self.read_error_pipe
            .read_to_end(&mut data)
            .map_err(|err| Error::DeserializeError(err.description().into()))?;
        let result = if data.len() > 0 {
            Some(bincode::deserialize(&data)
                .map_err(|err| Error::DeserializeError(err.description().into()))?)
        } else {
            None
        };

        loop {
            let usage = usage()?;
            for timeout in limits.wall_time() {
                if timer.elapsed() >= timeout {
                    return Ok(RunInfo::new(RunInfoResult::WallTimeLimitExceeded, usage));
                }
            }

            if let Some(run_info) = usage.check_limits(limits) {
                return Ok(run_info);
            }

            // Check if something killed us
            let mut status: libc::c_int = 0;
            match unsafe { libc::waitpid(self.pid, &mut status, 0) } {
                -1 => {
                    let error = errno::Errno::last_error();
                    if error.error_code() == libc::EINTR {
                        continue; // interrupted by some signal
                    }
                    return Err(Error::FFIError(FFIError::WaitPidError(
                        error.error_string(),
                    )));
                }
                _ => {
                    if unsafe { libc::WIFEXITED(status) } {
                        let exit_code = unsafe { libc::WEXITSTATUS(status) };
                        if exit_code == 0 {
                            return Ok(RunInfo::new(RunInfoResult::Success(result), usage));
                        } else {
                            return Ok(RunInfo::new(
                                RunInfoResult::NonZeroExitStatus(exit_code),
                                usage,
                            ));
                        }
                    }

                    if unsafe { libc::WIFSIGNALED(status) } {
                        let signal = unsafe { libc::WTERMSIG(status) };
                        return Ok(RunInfo::new(RunInfoResult::KilledBySignal(signal), usage));
                    }

                    if unsafe { libc::WIFSTOPPED(status) || libc::WIFCONTINUED(status) } {
                        return Err(Error::StoppedContinuedError);
                    }
                }
            }
        }
    }
}
