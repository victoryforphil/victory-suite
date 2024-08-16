use std::{sync::Arc, time::Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VicTimecode {
    secs: u64,
    nanos: u32,
}

impl Default for VicTimecode {
    fn default() -> Self {
        VicTimecode { secs: 0, nanos: 0 }
    }
}

impl VicTimecode {
    pub fn new_secs(secs: f64) -> VicTimecode {
        let secs = secs as f64;
        let nanos = (secs.fract() * 1_000_000_000.0) as u32;
        VicTimecode {
            secs: secs as u64,
            nanos,
        }
    }

    pub fn new(secs: u64, nanos: u32) -> VicTimecode {
        VicTimecode { secs, nanos }
    }

    pub fn zero() -> VicTimecode {
        VicTimecode { secs: 0, nanos: 0 }
    }

    pub fn new_hz(hz: f64) -> VicTimecode {
        let secs = 1.0 / hz;
        VicTimecode::new_secs(secs)
    }

    pub fn new_ms(ms: f64) -> VicTimecode {
        let secs = ms / 1000.0;
        VicTimecode::new_secs(secs)
    }

    pub fn new_us(us: f64) -> VicTimecode {
        let secs = us / 1_000_000.0;
        VicTimecode::new_secs(secs)
    }

    pub fn secs(&self) -> f64 {
        self.secs as f64 + (self.nanos as f64 / 1_000_000_000.0)
    }

    pub fn ms(&self) -> f64 {
        self.secs as f64 * 1000.0 + (self.nanos as f64 / 1_000_000.0)
    }

    pub fn us(&self) -> f64 {
        self.secs as f64 * 1_000_000.0 + (self.nanos as f64 / 1_000.0)
    }
}
pub type VicInstantHandle = Arc<VicInstant>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn now() -> VicInstant {
        let now = Instant::now();
        now.into()
    }

    pub fn handle(&self) -> VicInstantHandle {
        Arc::new(self.clone())
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
