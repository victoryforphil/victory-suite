use crate::Timepoint;

use super::Timecode;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timespan {
    pub time: Timecode,
}

impl Timespan {
    pub fn new(time_duration: Timecode) -> Timespan {
        Timespan { time: time_duration }
    }

    pub fn new_points(start: Timepoint, end: Timepoint) -> Timespan {
        end - start
    }

    pub fn new_secs(secs: f64) -> Timespan {
        Self::new(Timecode::new_secs(secs))
    }
    pub fn new_hz(hz: f64) -> Timespan {
        Self::new(Timecode::new_hz(hz))
    }

    pub fn new_ms(ms: f64) -> Timespan {
        Self::new(Timecode::new_ms(ms))
    }

    pub fn new_us(us: f64) -> Timespan {
        Self::new(Timecode::new_us(us))
    }

    pub fn from_duration(duration: std::time::Duration) -> Timespan {
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        Timespan {
            time: Timecode::new(secs, nanos),
        }
    }

    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::new(self.time.secs, self.time.nanos)
    }

    pub fn secs(&self) -> f64 {
        self.time.secs()
    }

    pub fn ms(&self) -> f64 {
        self.time.ms()
    }

    pub fn us(&self) -> f64 {
        self.time.us()
    }

    pub fn zero() -> Timespan {
        Timespan {
            time: Timecode::zero(),
        }
    }
}

impl From<std::time::Duration> for Timespan {
    fn from(duration: std::time::Duration) -> Self {
        Timespan::from_duration(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vic_duration() {
        let duration = Timespan::new_secs(1.0);
        assert_eq!(duration.time.secs, 1);
        assert_eq!(duration.time.nanos, 0);
        assert_eq!(duration.secs(), 1.0);
        assert_eq!(duration.ms(), 1000.0);
        assert_eq!(duration.us(), 1_000_000.0);
    }
}
