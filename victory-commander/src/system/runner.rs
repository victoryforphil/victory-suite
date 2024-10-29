use std::{sync::{Arc, Mutex}, time::Duration};

use log::{debug, info, warn};
use victory_broker::{adapters::{PubSubAdapter, PubSubAdapterHandle}, node::Node};
use victory_data_store::database::{DataView, Datastore};
use victory_wtf::{Timepoint, Timespan};

use super::{System, SystemHandle};

pub struct BasherSysRunner {
    pub systems: Vec<SystemHandle>,
    pub data_store: Arc<Mutex<Datastore>>,
    pub end_time: Timepoint,
    pub current_time: Timepoint,
    pub dt: Timespan,
    pub real_time: bool,
    pub pubsub_node: Option<Node>,
}

impl BasherSysRunner {
    pub fn new() -> BasherSysRunner {
        BasherSysRunner {
            systems: Vec::new(),
            data_store: Arc::new(Mutex::new(Datastore::new())),
            end_time: Timepoint::zero(),
            current_time: Timepoint::zero(),
            dt: Timespan::new_hz(100.0),
            real_time: false,
            pubsub_node: None,
        }
    }
    pub fn enable_pubsub(&mut self, adapter: PubSubAdapterHandle) {
        self.pubsub_node = Some(Node::new(
            "Commander PubSub Node".to_string(),
            adapter,
            self.data_store.clone(),
        ));
    }
    pub fn add_system(&mut self, system: SystemHandle) {
        self.systems.push(system);
    }

    pub fn set_real_time(&mut self, real_time: bool) {
        self.real_time = real_time;
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

            if let Some(node) = &mut self.pubsub_node {
                node.tick();
            }

            let start_time = Timepoint::now();
            for system in self.systems.iter_mut() {
                let mut system = system.lock().unwrap();
                let sub = system.get_subscribed_topics();

                let mut inputs = DataView::new();
                for topic in sub.iter() {
                    inputs = inputs.add_query(&self.data_store.lock().unwrap(), topic).unwrap();
                }

                let new_data = system.execute(&inputs, self.dt.clone());
                self.data_store.lock().unwrap().apply_view(new_data).unwrap();
            }
            let end_time = Timepoint::now();
            let elapsed = end_time - start_time;
            if let Some(node) = &mut self.pubsub_node {
                node.tick();
            }
            let sleep_time = self.dt.clone().secs() - elapsed.secs();
            if sleep_time < 0.0 {
                warn!("System is taking too long to run! {:?}s", sleep_time);
            } else {
                let sleep_duration = Duration::from_secs_f64(sleep_time);

                if self.real_time {
                    std::thread::sleep(sleep_duration);
                }
            }

            self.current_time = self.current_time.clone() + self.dt.clone();
            
        }
        info!("Finished running main loop");
    }
}
