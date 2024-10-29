use std::sync::{Arc, Mutex};

use log::info;
use system::runner::BasherSysRunner;
use test_systems::time_scale::{TimeScaleData, TimeScaleSystem};
use victory_data_store::topics::TopicKey;
use victory_wtf::Timepoint;
mod system;
mod test_systems;

pub fn main() {
    pretty_env_logger::init();
    info!("Hello from victory-commander!");
    let key = TopicKey::from_str("time_data");
    let mut runner = BasherSysRunner::new();
    runner.data_store.lock().unwrap().add_struct(
        &key.clone(),
        Timepoint::now(),
        TimeScaleData { test_value: 10.0 },
    );

    let time_system = TimeScaleSystem::new("time_data");
    runner.add_system(Arc::new(Mutex::new(time_system)));

    runner.run(Timepoint::new_secs(1000.0));

    for key in runner.data_store.lock().unwrap().get_all_keys() {
        info!("Key: {:?}", key);
        let latest = runner.data_store.lock().unwrap().get_latest_primitive(&key).unwrap();
        info!("Latest: {:?}", latest);
    }
}
