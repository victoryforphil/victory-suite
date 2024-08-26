use super::VicTimecode;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VicDuration {
    pub time: VicTimecode,
}

impl VicDuration {
    pub fn new(time: VicTimecode) -> VicDuration {
        VicDuration { time }
    }

    pub fn new_secs(secs: f64) -> VicDuration {
        Self::new(VicTimecode::new_secs(secs))
    }
    pub fn new_hz(hz: f64) -> VicDuration {
        Self::new(VicTimecode::new_hz(hz))
    }

    pub fn new_ms(ms: f64) -> VicDuration {
        Self::new(VicTimecode::new_ms(ms))
    }

    pub fn new_us(us: f64) -> VicDuration {
        Self::new(VicTimecode::new_us(us))
    }

    pub fn from_duration(duration: std::time::Duration) -> VicDuration {
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        VicDuration {
            time: VicTimecode::new(secs, nanos),
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

    pub fn zero() -> VicDuration {
        VicDuration {
            time: VicTimecode::zero(),
        }
    }
}

impl From<std::time::Duration> for VicDuration {
    fn from(duration: std::time::Duration) -> Self {
        VicDuration::from_duration(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vic_duration() {
        let duration = VicDuration::new_secs(1.0);
        assert_eq!(duration.time.secs, 1);
        assert_eq!(duration.time.nanos, 0);
        assert_eq!(duration.secs(), 1.0);
        assert_eq!(duration.ms(), 1000.0);
        assert_eq!(duration.us(), 1_000_000.0);
    }
}
