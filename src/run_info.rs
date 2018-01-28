use std::fmt::{self, Display, Formatter};
use std::time::Duration;

use config::{Limits, SpaceUsage};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RunInfoResult<T> {
    Success(T),
    NonZeroExitStatus(i32),
    KilledBySignal(i32),
    MemoryLimitExceeded,
    TimeLimitExceeded,
    WallTimeLimitExceeded,
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
            RunInfoResult::MemoryLimitExceeded => RunInfoResult::MemoryLimitExceeded,
            RunInfoResult::TimeLimitExceeded => RunInfoResult::TimeLimitExceeded,
            RunInfoResult::WallTimeLimitExceeded => RunInfoResult::WallTimeLimitExceeded,
        })
    }

    pub fn success(self) -> Option<T> {
        match self {
            RunInfoResult::Success(obj) => Some(obj),
            _ => None,
        }
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
            &RunInfoResult::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            &RunInfoResult::TimeLimitExceeded => write!(f, "Time limit exceeded"),
            &RunInfoResult::WallTimeLimitExceeded => write!(f, "Wall time limit exceeded"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunUsage {
    user_time: Duration,
    memory: SpaceUsage,
}

impl RunUsage {
    pub fn new(user_time: Duration, memory: SpaceUsage) -> RunUsage {
        RunUsage { user_time, memory }
    }

    pub fn user_time(&self) -> Duration {
        self.user_time
    }

    pub fn memory(&self) -> SpaceUsage {
        self.memory
    }

    pub fn check_limits<T>(self, limits: Limits) -> Option<RunInfo<T>> {
        if limits
            .user_time()
            .map(|time| time < self.user_time())
            .unwrap_or(false)
        {
            return Some(RunInfo::new(RunInfoResult::TimeLimitExceeded, self));
        }

        if limits
            .memory()
            .map(|memory| memory < self.memory())
            .unwrap_or(false)
        {
            return Some(RunInfo::new(RunInfoResult::MemoryLimitExceeded, self));
        }

        None
    }
}

impl Default for RunUsage {
    fn default() -> RunUsage {
        RunUsage::new(Duration::from_secs(0), SpaceUsage::from_bytes(0))
    }
}

impl Display for RunUsage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "Total user time: {}",
            self.user_time().as_secs() as f64
                + (self.user_time().subsec_nanos() as f64) / 1_000_000_000.
        )?;
        write!(f, "Total memory: {}", self.memory())
    }
}
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunInfo<T> {
    result: RunInfoResult<T>,
    usage: RunUsage,
}

impl<T> RunInfo<T> {
    pub fn new(result: RunInfoResult<T>, usage: RunUsage) -> RunInfo<T> {
        RunInfo { result, usage }
    }

    pub fn result(&self) -> &RunInfoResult<T> {
        &self.result
    }

    pub fn usage(&self) -> &RunUsage {
        &self.usage
    }

    pub fn is_success(&self) -> bool {
        self.result.is_success()
    }

    pub fn and_then<A, B, F: FnOnce(T) -> Result<A, B>>(self, cb: F) -> Result<RunInfo<A>, B> {
        let RunInfo { result, usage } = self;
        result
            .and_then(cb)
            .map(|result| RunInfo::new(result, usage))
    }

    pub fn success(self) -> Option<T> {
        self.result.success()
    }
}

impl<T> Display for RunInfo<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{}", self.result)?;
        write!(f, "{}", self.usage)
    }
}
