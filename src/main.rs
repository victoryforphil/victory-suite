use enum_dispatch::enum_dispatch;
use log::info;
use strum_macros::EnumIter;
use system::{mock_system::MockSystem, runner::BasherSysRunner};
use victory_time_rs::Timepoint;
mod system;
mod mock_system;
#[enum_dispatch(System)]
#[derive(EnumIter)]
enum MySystems{
    MockSystem 
}

pub fn main() {
    pretty_env_logger::init();
    info!("Hello from victory-commander!");

    let mut runner = BasherSysRunner::<MySystems>::new();
    runner.run(Timepoint::new_secs(10.0));
    info!("Finished");
    let keys = runner.data_store.get_keys();
    info!("Keys: {:?}", keys);
}


