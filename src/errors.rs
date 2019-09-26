use std::ffi::OsString;
use std::path::PathBuf;
use std::result::Result as StdResult;

#[derive(Fail, Debug, Serialize, Deserialize)]
pub enum FFIError {
    #[fail(display = "Could not chdir to {:?}: {}", path, error)]
    ChdirError { path: PathBuf, error: String },
    #[fail(display = "Could not chroot to {:?}: {}", path, error)]
    ChrootError { path: PathBuf, error: String },
    #[fail(display = "Could not clone process: {}", _0)]
    CloneError(String),
    #[fail(display = "Could not dup file descriptor {}({}): {}", name, fd, error)]
    DupFdError {
        fd: i32,
        name: String,
        error: String,
    },
    #[fail(display = "Could not create directory {:?}: {}", path, error)]
    CreateDirError { path: PathBuf, error: String },
    #[fail(
        display = "Could not exec {:?} (arguments: {:?}): {}",
        command, arguments, error
    )]
    ExecError {
        command: PathBuf,
        arguments: Vec<OsString>,
        error: String,
    },
    #[fail(display = "Could not mount path: {:?}: {}", path, error)]
    MountError { path: PathBuf, error: String },
    #[fail(display = "Could not open file descriptor {}({}): {}", name, fd, error)]
    OpenFdError {
        fd: i32,
        name: String,
        error: String,
    },
    #[fail(display = "Could not create pipe: {}", _0)]
    Pipe2Error(String),
    #[fail(
        display = "Could not pivot_root to {:?} with old root at {:?}: {}",
        new_root, old_root, error
    )]
    PivotRootError {
        new_root: PathBuf,
        old_root: PathBuf,
        error: String,
    },
    #[fail(display = "Could not set process to die when parent dies: {}", _0)]
    PrSetPDeathSigError(String),
    #[fail(display = "Could not set interval timer alarm: {}", _0)]
    SetITimerError(String),
    #[fail(
        display = "Could not set process group id of {} to {}: {}",
        pid, pgid, error
    )]
    SetpgidError { pid: i32, pgid: i32, error: String },
    #[fail(display = "Could not set resource limit: {}", _0)]
    SetRLimitError(String),
    #[fail(display = "Could not set a signal handler for {}: {}", signal, error)]
    SigActionError { signal: String, error: String },
    #[fail(display = "Could not umount path: {:?}: {}", path, error)]
    UMountError { path: PathBuf, error: String },
    #[fail(display = "Could not unshare cgroup namespace: {}", _0)]
    UnshareCGroupError(String),
    #[fail(display = "Could not usleep for {} microseconds: {}", time, error)]
    UsleepError { time: u32, error: String },
    #[fail(display = "Could not write /proc/self/uid_map file: {}", _0)]
    WriteUidError(String),
    #[fail(display = "Could not write /proc/self/uid_map file: {}", _0)]
    WriteGidError(String),
    #[fail(display = "Could not wait for process: {}", _0)]
    WaitPidError(String),
    #[fail(display = "Could not write /proc/self/setgroups file: {}", _0)]
    WriteSetGroupsError(String),
}

#[derive(Fail, Debug, Serialize, Deserialize)]
pub enum CGroupError {
    #[fail(display = "Cgroup controller missing: {:?}", _0)]
    ControllerMissing(PathBuf),
    #[fail(
        display = "Could not create instance controller under {:?} for {:?}: {}",
        controller_path, instance_name, error
    )]
    InstanceControllerCreateError {
        controller_path: PathBuf,
        instance_name: OsString,
        error: String,
    },
    #[fail(
        display = "Could not open {:?} for controller {:?}: {}",
        file, controller_path, error
    )]
    OpenCGroupFileError {
        controller_path: PathBuf,
        file: PathBuf,
        error: String,
    },
    #[fail(
        display = "Could not parse `{}` from {:?} for controller {:?}: {}",
        buffer, file, controller_path, error
    )]
    ParseCGroupFileError {
        controller_path: PathBuf,
        file: PathBuf,
        buffer: String,
        error: String,
    },

    #[fail(
        display = "Could not read from {:?} for controller {:?}: {}",
        file, controller_path, error
    )]
    ReadCGroupFileError {
        controller_path: PathBuf,
        file: PathBuf,
        error: String,
    },
    #[fail(
        display = "Could not write to {:?} for controller {:?}: {}",
        file, controller_path, error
    )]
    WriteCGroupFileError {
        controller_path: PathBuf,
        file: PathBuf,
        error: String,
    },
}

#[derive(Fail, Debug, Serialize, Deserialize)]
pub enum ChildError {
    #[fail(display = "Cgroup error occurred.")]
    CGroupError(#[cause] CGroupError),
    #[fail(display = "FFI Error occurred.")]
    FFIError(#[cause] FFIError),
}

impl From<CGroupError> for ChildError {
    fn from(err: CGroupError) -> Self {
        Self::CGroupError(err)
    }
}

impl From<FFIError> for ChildError {
    fn from(err: FFIError) -> Self {
        Self::FFIError(err)
    }
}

#[derive(Fail, Debug, Serialize, Deserialize)]
pub enum Error {
    #[fail(display = "Cgroup error occurred.")]
    CGroupError(#[cause] CGroupError),
    #[fail(display = "Child process error occurred.")]
    ChildError(#[cause] ChildError),
    #[fail(display = "Child process successfully completed even though it used exec")]
    ContinuedPastExecError(String),
    #[fail(display = "Could not deserialize process result: {}", _0)]
    DeserializeError(String),
    #[fail(display = "FFI Error occurred.")]
    FFIError(#[cause] FFIError),
    #[fail(display = "Child process stopped/continued unexpected")]
    StoppedContinuedError,
    #[fail(display = "Supervisor process died and could not collect execution information")]
    SupervisorProcessDiedError,
}

impl From<CGroupError> for Error {
    fn from(err: CGroupError) -> Self {
        Self::CGroupError(err)
    }
}

impl From<ChildError> for Error {
    fn from(err: ChildError) -> Self {
        Self::ChildError(err)
    }
}

impl From<FFIError> for Error {
    fn from(err: FFIError) -> Self {
        Self::FFIError(err)
    }
}

pub type Result<T> = StdResult<T, Error>;
