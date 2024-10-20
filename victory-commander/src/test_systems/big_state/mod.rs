use std::collections::HashMap;
pub mod big_pub;
pub mod big_sub;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BigStateVector{
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: Option<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BigStatePose{
    pub position: BigStateVector,
    pub orientation: BigStateVector,
    pub linear_velocity: BigStateVector,
    pub angular_velocity: BigStateVector,
    pub linear_acceleration: BigStateVector,
    pub angular_acceleration: BigStateVector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigState{
    pub pose: BigStatePose,
    pub trajectory: HashMap<String, BigStatePose>,
}

impl BigState{
    pub fn new() -> BigState{
        let mut trajectory = HashMap::new();
        for i in 0..10{
            trajectory.insert(i.to_string(), BigStatePose::default());
        }
        BigState{
            pose: BigStatePose::default(),
            trajectory: trajectory,
        }
    }
}


#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use big_pub::BigStatePublisher;
    use big_sub::BigStateSubscriber;
    use victory_time_rs::{Timepoint, Timespan};

    use crate::system::runner::BasherSysRunner;

    use super::*;

    #[test]
    fn test_big_state() {
        pretty_env_logger::init();
        let mut runner = BasherSysRunner::new();
        let big_pub = BigStatePublisher::new();
        let big_sub = BigStateSubscriber::new();
        runner.add_system(Arc::new(Mutex::new(big_pub)));
        runner.add_system(Arc::new(Mutex::new(big_sub)));
        let now = Timepoint::now();
        runner.dt = Timespan::new_hz(100.0);
        runner.run(Timepoint::new_secs(10.0));
        let end = Timepoint::now();
        let duration = end - now;
        println!("{:?}", duration.secs());

        assert!(duration.secs() < 20.0);
        
        
    }
}