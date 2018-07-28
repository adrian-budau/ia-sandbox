use std::fmt::{self, Display, Formatter};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct DurationDisplay(pub Duration);

impl Display for DurationDisplay {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let nanos = self.0.as_secs() * 1_000_000_000u64 + u64::from(self.0.subsec_nanos());
        if nanos < 1000 {
            write!(formatter, "{}ns", nanos)
        } else if nanos < 10_000_000 {
            write!(formatter, "{:.3}us", (nanos as f64) / 1_000.)
        } else if nanos < 1_000_000_000 {
            write!(formatter, "{:.3}ms", (nanos as f64) / 1_000_000.)
        } else {
            write!(formatter, "{:.3}s", (nanos as f64) / 1_000_000_000.)
        }
    }
}

pub trait DurationExt {
    fn as_milliseconds(&self) -> u128;
}

impl DurationExt for Duration {
    fn as_milliseconds(&self) -> u128 {
        u128::from(self.as_secs()) * 1000 + u128::from(self.subsec_nanos()) / 1_000_000
    }
}
