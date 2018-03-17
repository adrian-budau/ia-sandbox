use std::fmt::{self, Display, Formatter};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct DurationDisplay(pub Duration);

impl Display for DurationDisplay {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let nanos = self.0.as_secs() * 1_000_000_000u64 + self.0.subsec_nanos() as u64;
        if nanos < 1000 {
            write!(formatter, "{}ns", nanos)
        } else if nanos < 10000000 {
            write!(formatter, "{:.3}us", (nanos as f64) / 1_000.)
        } else if nanos < 1000000000 {
            write!(formatter, "{:.3}ms", (nanos as f64) / 1_000_000.)
        } else {
            write!(formatter, "{:.3}s", (nanos as f64) / 1_000_000_000.)
        }
    }
}

pub trait DurationExt {
    fn as_millis(&self) -> u64;
}

impl DurationExt for Duration {
    fn as_millis(&self) -> u64 {
        self.as_secs() * 1000 + self.subsec_nanos() as u64 / 1_000_000
    }
}
