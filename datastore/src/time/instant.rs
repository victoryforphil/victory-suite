use std::{ops::Add, sync::Arc, time::Instant};

use super::{VicDuration, VicTimecode};
use serde::{Deserialize, Serialize};
pub type VicInstantHandle = Arc<VicInstant>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VicInstant {
    pub time: VicTimecode,
}
impl Default for VicInstant {
    fn default() -> Self {
        VicInstant {
            time: VicTimecode::zero(),
        }
    }
}

impl VicInstant {
    pub fn new(time: VicTimecode) -> VicInstant {
        VicInstant { time }
    }
    pub fn new_secs(secs: f64) -> VicInstant {
        Self::new(VicTimecode::new_secs(secs))
    }

    pub fn new_ms(ms: f64) -> VicInstant {
        Self::new(VicTimecode::new_ms(ms))
    }

    pub fn new_us(us: f64) -> VicInstant {
        Self::new(VicTimecode::new_us(us))
    }

    pub fn now() -> VicInstant {
        let now = Instant::now();
        now.into()
    }

    pub fn handle(&self) -> VicInstantHandle {
        Arc::new(self.clone())
    }

    pub fn zero() -> VicInstant {
        VicInstant {
            time: VicTimecode::zero(),
        }
    }

    pub fn secs(&self) -> f64 {
        self.time.secs()
    }

    pub fn ms(&self) -> f64 {
        self.time.ms()
    }
}

impl Add<VicDuration> for VicInstant {
    type Output = VicInstant;

    fn add(self, rhs: VicDuration) -> Self::Output {
        let duration_time = rhs.time;
        let new_time = VicTimecode {
            secs: self.time.secs + duration_time.secs,
            nanos: self.time.nanos + duration_time.nanos,
        };
        VicInstant::new(new_time)
    }
}

impl From<Instant> for VicInstant {
    fn from(instant: Instant) -> Self {
        let duration = instant.elapsed();
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        VicInstant {
            time: VicTimecode::new(secs, nanos),
        }
    }
}

impl From<Arc<VicInstant>> for VicInstant {
    fn from(handle: Arc<VicInstant>) -> Self {
        handle.as_ref().clone()
    }
}

impl From<VicTimecode> for VicInstant {
    fn from(time: VicTimecode) -> Self {
        VicInstant { time }
    }
}
