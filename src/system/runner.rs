
use log::{debug, info};
use victory_data_store::database::{DataView, Datastore};
use victory_time_rs::{Timepoint, Timespan};

use super::{System, SystemHandle};

pub struct BasherSysRunner {
    pub systems: Vec<SystemHandle>,
    pub data_store: Datastore,
    pub end_time: Timepoint,
    pub current_time: Timepoint,
    pub dt: Timespan,
}

impl BasherSysRunner {
    pub fn new() -> BasherSysRunner {
        BasherSysRunner {
            systems: Vec::new(),
            data_store: Datastore::new(),
            end_time: Timepoint::zero(),
            current_time: Timepoint::zero(),
            dt: Timespan::new_hz(100.0),
        }
    }

    pub fn add_system(&mut self, system: SystemHandle) {
        self.systems.push(system);
    }

    pub fn run(&mut self, end_time: Timepoint) {
        self.end_time = end_time;
        self.current_time = Timepoint::zero();

        for system in self.systems.iter_mut() {
            log::info!("Initializing system: {:?}", system.lock().unwrap().name());
            system.lock().unwrap().init();
        }
        info!("Running main loop for {:?}s", self.end_time.secs());
        while self.current_time < self.end_time {
            for system in self.systems.iter_mut() {
                let mut system = system.lock().unwrap();
                let sub = system.get_subscribed_topics();
                
              
                let mut inputs = DataView::new();
                for topic in sub.iter() {
                    inputs = inputs.add_query(&self.data_store, topic).unwrap();
                }
           
                let new_data = system.execute(&inputs, self.dt.clone());
                self.data_store.apply_view(new_data).unwrap();
            }
            self.current_time = self.current_time.clone() + self.dt.clone();
        }
        info!("Finished running main loop");
    }
}
