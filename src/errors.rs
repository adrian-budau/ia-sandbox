use std::ffi::OsString;
use std::path::PathBuf;
use std::result::Result as StdResult;

#[derive(Fail, Debug, Serialize, Deserialize)]
pub enum FFIError {
    #[fail(display = "Could not chdir to {:?}: {}", path, error)]
    ChdirError {
        path: PathBuf,
        error: String,
    },
    #[fail(display = "Could not chroot to {:?}: {}", path, error)]
    ChrootError {
        path: PathBuf,
        error: String,
    },
    #[fail(display = "Could not clone process: {}", _0)] CloneError(String),
    #[fail(display = "Could not close file descriptor {}({}): {}", name, fd, error)]
    CloseFdError {
        fd: i32,
        name: String,
        error: String,
    },
    #[fail(display = "Could not create directory {:?}: {}", path, error)]
    CreateDirError {
        path: PathBuf,
        error: String,
    },
    #[fail(display = "Could not exec {:?} (arguments: {:?}): {}", command, arguments, error)]
    ExecError {
        command: PathBuf,
        arguments: Vec<OsString>,
        error: String,
    },
    #[fail(display = "Could not mount path: {:?}: {}", path, error)]
    MountError {
        path: PathBuf,
        error: String,
    },
    #[fail(display = "Could not open file descriptor {}({}): {}", name, fd, error)]
    OpenFdError {
        fd: i32,
        name: String,
        error: String,
    },
    #[fail(display = "Could not create pipe: {}", _0)] Pipe2Error(String),
    #[fail(display = "Could not pivot_root to {:?} with old root at {:?}: {}", new_root,
           old_root, error)]
    PivotRootError {
        new_root: PathBuf,
        old_root: PathBuf,
        error: String,
    },
    #[fail(display = "Could not set process to die when parent dies: {}", _0)]
    PrSetPDeathSigError(String),
    #[fail(display = "Could not set interval timer alarm: {}", _0)] SetITimerError(String),
    #[fail(display = "Could not set process group id of {} to {}: {}", pid, pgid, error)]
    SetpgidError {
        pid: i32,
        pgid: i32,
        error: String,
    },
    #[fail(display = "Could not set a signal handler for {}: {}", signal, error)]
    SigActionError {
        signal: String,
        error: String,
    },
    #[fail(display = "Could not umount path: {:?}: {}", path, error)]
    UMountError {
        path: PathBuf,
        error: String,
    },
    #[fail(display = "Could not usleep for {} microseconds: {}", time, error)]
    UsleepError {
        time: u32,
        error: String,
    },
    #[fail(display = "Could not write /proc/self/uid_map file: {}", _0)] WriteUidError(String),
    #[fail(display = "Could not write /proc/self/uid_map file: {}", _0)] WriteGidError(String),
    #[fail(display = "Could not wait for process: {}", _0)] WaitPidError(String),
    #[fail(display = "Could not write /proc/self/setgroups file: {}", _0)]
    WriteSetGroupsError(String),
}

#[derive(Fail, Debug, Serialize, Deserialize)]
pub enum Error {
    #[fail(display = "Child process error occurred.")] ChildError(#[cause] FFIError),
    #[fail(display = "Child process successfully completed even though it used exec")]
    ContinuedPastExecError(String),
    #[fail(display = "Could not deserialize process result: {}", _0)] DeserializeError(String),
    #[fail(display = "FFI Error occurred.")] FFIError(#[cause] FFIError),
    #[fail(display = "Child process stopped/continued unexpected")] StoppedContinuedError,
    #[fail(display = "Supervisor process died and could not collect execution information")]
    SupervisorProcessDiedError,
}

pub type Result<T> = StdResult<T, Error>;
