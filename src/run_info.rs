use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum RunInfoResult<T> {
    Success(T),
    NonZeroExitStatus(i32),
    KilledBySignal(i32),
}

impl<T> RunInfoResult<T> {
    pub fn is_success(&self) -> bool {
        match self {
            &RunInfoResult::Success(_) => true,
            _ => false,
        }
    }

    pub fn and_then<A, B, F: FnOnce(T) -> Result<A, B>>(
        self,
        cb: F,
    ) -> Result<RunInfoResult<A>, B> {
        Ok(match self {
            RunInfoResult::Success(obj) => RunInfoResult::Success(cb(obj)?),
            RunInfoResult::NonZeroExitStatus(exit_status) => {
                RunInfoResult::NonZeroExitStatus(exit_status)
            }
            RunInfoResult::KilledBySignal(signal) => RunInfoResult::KilledBySignal(signal),
        })
    }
}

impl<T> Display for RunInfoResult<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &RunInfoResult::Success(_) => write!(f, "Success"),
            &RunInfoResult::NonZeroExitStatus(ref exit_code) => {
                write!(f, "Non zero exit status: {}", exit_code)
            }
            &RunInfoResult::KilledBySignal(ref signal) => write!(f, "Killed by Signal {}", signal),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunInfo<T> {
    result: RunInfoResult<T>,
}

impl<T> RunInfo<T> {
    pub fn new(result: RunInfoResult<T>) -> RunInfo<T> {
        RunInfo { result }
    }

    pub fn result(&self) -> &RunInfoResult<T> {
        &self.result
    }

    pub fn is_success(&self) -> bool {
        self.result.is_success()
    }

    pub fn and_then<A, B, F: FnOnce(T) -> Result<A, B>>(self, cb: F) -> Result<RunInfo<A>, B> {
        self.result.and_then(cb).map(RunInfo::new)
    }
}

impl<T> Display for RunInfo<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.result)
    }
}
