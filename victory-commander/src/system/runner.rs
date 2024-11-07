use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, info, warn};
use victory_broker::{
    adapters::{PubSubAdapter, PubSubAdapterHandle},
    node::{sub_callback::SubCallback, Node},
};
use victory_data_store::{
    buckets::BucketHandle,
    database::{DataView, Datastore},
    topics::{TopicKeyHandle, TopicKeyProvider},
};
use victory_wtf::{Timepoint, Timespan};

use super::{System, SystemHandle};

pub struct RunnerPubSubCallback {
    bucket: BucketHandle,
}

impl RunnerPubSubCallback {
    pub fn new(bucket: BucketHandle) -> RunnerPubSubCallback {
        debug!(
            "RunnerPubSubCallback::new: Creating new RunnerPubSubCallback for bucket {:?}",
            bucket
        );
        RunnerPubSubCallback { bucket }
    }
}

impl SubCallback for RunnerPubSubCallback {
    fn on_update(&mut self, datapoints: &victory_data_store::datapoints::DatapointMap) {
        for (key, value) in datapoints.iter() {
            self.bucket.write().unwrap().update_datapoint(value.clone());
        }
    }
}

pub type RunnerPubSubCallbackHandle = Arc<Mutex<dyn SubCallback + Send + Sync>>;
pub struct BasherSysRunner {
    pub systems: Vec<SystemHandle>,
    pub data_store: Arc<Mutex<Datastore>>,
    pub end_time: Option<Timepoint>,
    pub current_time: Timepoint,
    pub dt: Timespan,
    pub real_time: bool,
    pub pubsub_node: Option<Node>,
    pub pubsub_callbacks: BTreeMap<TopicKeyHandle, RunnerPubSubCallbackHandle>,
}

impl BasherSysRunner {
    pub fn new() -> BasherSysRunner {
        BasherSysRunner {
            systems: Vec::new(),
            data_store: Arc::new(Mutex::new(Datastore::new())),
            end_time: None,
            current_time: Timepoint::zero(),
            dt: Timespan::new_hz(100.0),
            real_time: false,
            pubsub_node: None,
            pubsub_callbacks: BTreeMap::new(),
        }
    }

    pub fn set_end_time(&mut self, end_time: Timepoint) {
        self.end_time = Some(end_time);
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

    pub fn run(&mut self) {
        self.current_time = Timepoint::zero();

        for system in self.systems.iter_mut() {
            log::info!("Initializing system: {:?}", system.lock().unwrap().name());
            system.lock().unwrap().init();
        }

        // If we have a pubsub node, subscribe to all topics
        if let Some(node) = &mut self.pubsub_node {
            for system in self.systems.iter_mut() {
                let sub = system.lock().unwrap().get_subscribed_topics();
                for topic in sub.iter() {
                    debug!("Adding pubsub runner callback for topic {:?}", topic);
                    let bucket = self.data_store.lock().unwrap().get_or_create_bucket(topic);
                    let callback = RunnerPubSubCallback::new(bucket);
                    let callback = Arc::new(Mutex::new(callback));
                    self.pubsub_callbacks
                        .insert(topic.handle().clone(), callback.clone());
                    node.add_sub_callback(topic.handle(), callback);
                }
            }
        }

        let end_time = match &self.end_time {
            Some(end_time) => {
                info!("Running main loop for {:?}", end_time);
                end_time.clone()
            }
            None => {
                info!("Running main loop indefinitely");
                Timepoint::now() + Timespan::new_hz(100.0)
            }
        };
        while self.current_time < end_time {
            if let Some(node) = &mut self.pubsub_node {
                node.tick();
            }

            let start_time = Timepoint::now();
            for system in self.systems.iter_mut() {
                let mut system = system.lock().unwrap();
                let sub = system.get_subscribed_topics();

                let mut inputs = DataView::new();
                for topic in sub.iter() {
                    inputs = inputs
                        .add_query(&self.data_store.lock().unwrap(), topic)
                        .unwrap();
                }

                let new_data = system.execute(&inputs, self.dt.clone());
                self.data_store
                    .lock()
                    .unwrap()
                    .apply_view(new_data)
                    .unwrap();
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
