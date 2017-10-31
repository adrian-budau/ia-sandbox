use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum RunInfoResult {
    Success,
    NonZeroExitStatus(i32),
    KilledBySignal(i32),
}

impl RunInfoResult {
    pub fn is_success(&self) -> bool {
        match self {
            &RunInfoResult::Success => true,
            _ => false,
        }
    }
}

impl Display for RunInfoResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &RunInfoResult::Success => write!(f, "Success"),
            &RunInfoResult::NonZeroExitStatus(ref exit_code) => {
                write!(f, "Non zero exit status: {}", exit_code)
            }
            &RunInfoResult::KilledBySignal(ref signal) => write!(f, "Killed by Signal {}", signal),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RunInfo {
    result: RunInfoResult,
}

impl RunInfo {
    pub fn new(result: RunInfoResult) -> RunInfo {
        RunInfo { result }
    }

    pub fn result(&self) -> &RunInfoResult {
        &self.result
    }

    pub fn is_success(&self) -> bool {
        self.result.is_success()
    }
}

impl Display for RunInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.result)
    }
}
