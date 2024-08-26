use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VicTimecode {
    pub secs: u64,
    pub nanos: u32,
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
