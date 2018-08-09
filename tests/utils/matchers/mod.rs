use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

use ia_sandbox::config::Limits;
use ia_sandbox::run_info::{RunInfo, RunInfoResult};
use ia_sandbox::utils::DurationDisplay;

pub trait Matcher {
    type AssertionString: Display;
    type Output: Display;

    fn assertion_string(&self) -> Self::AssertionString;
    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output>;
}

pub struct IsSuccess;

impl Matcher for IsSuccess {
    type AssertionString = &'static str;
    type Output = RunInfo<()>;

    fn assertion_string(&self) -> Self::AssertionString {
        "result is Success"
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        if run_info.is_success() {
            Ok(())
        } else {
            Err(run_info)
        }
    }
}

#[derive(Clone, Copy)]
pub struct NonZeroExitStatus(Option<u32>);

impl NonZeroExitStatus {
    pub fn new(exit_status: u32) -> NonZeroExitStatus {
        NonZeroExitStatus(Some(exit_status))
    }

    pub fn any() -> NonZeroExitStatus {
        NonZeroExitStatus(None)
    }
}

impl Matcher for NonZeroExitStatus {
    type AssertionString = Cow<'static, str>;
    type Output = RunInfo<()>;

    fn assertion_string(&self) -> Self::AssertionString {
        match self.0 {
            Some(ref x) => Cow::Owned(format!("result is NonZeroExitStatus({})", x)),
            None => Cow::Borrowed("result is NonZeroExitStatus(_)"),
        }
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        match *run_info.result() {
            RunInfoResult::NonZeroExitStatus(x) if self.0.map(|y| x == y).unwrap_or(true) => Ok(()),
            _ => Err(run_info),
        }
    }
}

#[derive(Clone, Copy)]
pub struct KilledBySignal(pub u32);

impl Matcher for KilledBySignal {
    type AssertionString = String;
    type Output = RunInfo<()>;

    fn assertion_string(&self) -> Self::AssertionString {
        format!("result is KilledBySignal({})", self.0)
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        match *run_info.result() {
            RunInfoResult::KilledBySignal(x) if x == self.0 => Ok(()),
            _ => Err(run_info),
        }
    }
}

pub struct WallTimeLimitExceeded;

impl Matcher for WallTimeLimitExceeded {
    type AssertionString = &'static str;
    type Output = RunInfo<()>;

    fn assertion_string(&self) -> Self::AssertionString {
        "result is WallTimeLimitExceeded"
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        match *run_info.result() {
            RunInfoResult::WallTimeLimitExceeded => Ok(()),
            _ => Err(run_info),
        }
    }
}

pub struct TimeLimitExceeded;

impl Matcher for TimeLimitExceeded {
    type AssertionString = &'static str;
    type Output = RunInfo<()>;

    fn assertion_string(&self) -> Self::AssertionString {
        "result is TimeLimitExceeded"
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        match *run_info.result() {
            RunInfoResult::TimeLimitExceeded => Ok(()),
            _ => Err(run_info),
        }
    }
}

pub struct MemoryLimitExceeded;

impl Matcher for MemoryLimitExceeded {
    type AssertionString = &'static str;
    type Output = RunInfo<()>;

    fn assertion_string(&self) -> Self::AssertionString {
        "result is MemoryLimitExceeded"
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        match *run_info.result() {
            RunInfoResult::MemoryLimitExceeded => Ok(()),
            _ => Err(run_info),
        }
    }
}

pub struct AnnotateAssert<T: Matcher> {
    matcher: T,
    annotate: Cow<'static, str>,
}

impl<T: Matcher> AnnotateAssert<T> {
    pub fn new<A: Into<Cow<'static, str>>>(matcher: T, annotate: A) -> Self {
        Self {
            matcher,
            annotate: annotate.into(),
        }
    }
}

impl<T: Matcher> Matcher for AnnotateAssert<T> {
    type AssertionString = String;
    type Output = <T as Matcher>::Output;

    fn assertion_string(&self) -> Self::AssertionString {
        format!("{}: {}", self.annotate, self.matcher.assertion_string())
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        self.matcher.try_match(run_info)
    }
}

pub struct CompareLimits<T: Matcher> {
    matcher: T,
    limits: Limits,
}

impl<T: Matcher> CompareLimits<T> {
    pub fn new<L: Into<Limits>>(matcher: T, limits: L) -> CompareLimits<T> {
        CompareLimits {
            matcher,
            limits: limits.into(),
        }
    }
}

impl<T: Matcher> Matcher for CompareLimits<T> {
    type AssertionString = <T as Matcher>::AssertionString;
    type Output = CompareLimitsRunUsage;

    fn assertion_string(&self) -> Self::AssertionString {
        self.matcher.assertion_string()
    }

    fn try_match(&self, run_info: RunInfo<()>) -> Result<(), Self::Output> {
        self.matcher
            .try_match(run_info)
            .map_err(|_| CompareLimitsRunUsage(self.limits, run_info))
    }
}

pub struct CompareLimitsRunUsage(Limits, RunInfo<()>);

impl Display for CompareLimitsRunUsage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "Verdict: {}", self.1.result())?;
        if let Some(user_time_limit) = self.0.user_time() {
            writeln!(
                formatter,
                "User time: {} of maximum allowed {}",
                DurationDisplay(self.1.usage().user_time()),
                DurationDisplay(user_time_limit)
            )?;
        } else {
            writeln!(
                formatter,
                "User time: {}",
                DurationDisplay(self.1.usage().user_time())
            )?;
        }

        if let Some(wall_time_limit) = self.0.wall_time() {
            writeln!(
                formatter,
                "Wall time: {} of maximum allowed {}",
                DurationDisplay(self.1.usage().wall_time()),
                DurationDisplay(wall_time_limit)
            )?;
        } else {
            writeln!(
                formatter,
                "Wall time: {}",
                DurationDisplay(self.1.usage().wall_time())
            )?;
        }

        if let Some(memory_limit) = self.0.memory() {
            writeln!(
                formatter,
                "Memory usage: {} of maximum allowed {}",
                self.1.usage().memory(),
                memory_limit
            )?;
        } else {
            writeln!(formatter, "Memory usage: {}", self.1.usage().memory())?;
        }

        if let Some(stack_limit) = self.0.stack() {
            writeln!(formatter, "Maximum stack memory: {}", stack_limit)?;
        }

        if let Some(pids_limit) = self.0.pids() {
            writeln!(formatter, "Maximum pids allowed: {}", pids_limit)?;
        }

        Ok(())
    }
}
