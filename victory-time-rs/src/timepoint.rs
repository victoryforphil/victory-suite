use std::{ops::{Add, Sub}, sync::Arc, time::{Instant, SystemTime, UNIX_EPOCH}};

use crate::timespan::Timespan;

use super::Timecode;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timepoint {
    pub time: Timecode,
}
impl Default for Timepoint {
    fn default() -> Self {
        Timepoint {
            time: Timecode::zero(),
        }
    }
}

impl Timepoint {
    pub fn new(time: Timecode) -> Timepoint {
        Timepoint { time }
    }
    pub fn new_secs(secs: f64) -> Timepoint {
        Self::new(Timecode::new_secs(secs))
    }

    pub fn new_ms(ms: f64) -> Timepoint {
        Self::new(Timecode::new_ms(ms))
    }

    pub fn new_us(us: f64) -> Timepoint {
        Self::new(Timecode::new_us(us))
    }

    pub fn new_ns(ns: u128) -> Timepoint {
        Self::new(Timecode::new_ns(ns))
    }

    pub fn now() -> Timepoint {
        let now_nano = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_nanos(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        Timepoint::new(Timecode::new_ns(now_nano))

    }

    pub fn zero() -> Timepoint {
        Timepoint {
            time: Timecode::zero(),
        }
    }

    pub fn secs(&self) -> f64 {
        self.time.secs()
    }

    pub fn ms(&self) -> f64 {
        self.time.ms()
    }
}

impl Add<Timespan> for Timepoint {
    type Output = Timepoint;

    fn add(self, rhs: Timespan) -> Self::Output {
        let duration_time = rhs.time;
        let new_time = Timecode {
            secs: self.time.secs + duration_time.secs,
            nanos: self.time.nanos + duration_time.nanos,
        };
        Timepoint::new(new_time)
    }
}

/// Subtract operation.
/// Timepoint - Timepoint = Timespan
impl Sub<Timepoint> for Timepoint {
    type Output = Timespan;

    fn sub(self, rhs: Timepoint) -> Self::Output {
        let duration_time = self.time - rhs.time;

        Timespan::new(duration_time)
    }
}

impl From<Timecode> for Timepoint {
    fn from(time: Timecode) -> Self {
        Timepoint { time }
    }
}

#[cfg(test)]
mod tests_constructors {
    use super::*;

    #[test]
    fn test_new() {
        let time = Timecode::new(1, 0);
        let timepoint = Timepoint::new(time);
        assert_eq!(timepoint.time, time);
    }


}