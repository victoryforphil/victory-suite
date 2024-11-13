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
    database::{view::DataView, Datastore},
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
    pub fn enable_pubsub(&mut self, adapter: PubSubAdapterHandle) {}
    pub fn add_system(&mut self, system: SystemHandle) {
        self.systems.push(system);
    }

    pub fn set_real_time(&mut self, real_time: bool) {
        self.real_time = real_time;
    }

    #[tracing::instrument(skip_all)]
    pub fn run(&mut self) {
        let span = tracing::info_span!("runner_init");
        let _init_guard = span.enter();

        self.current_time = Timepoint::zero();

        for system in self.systems.iter_mut() {
            log::info!("Initializing system: {:?}", system.lock().unwrap().name());
            system.lock().unwrap().init();
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
        drop(_init_guard);

        let main_span = tracing::info_span!("main_loop");
        let _main_guard = main_span.enter();

        while self.current_time < end_time {
            let start_time = Timepoint::now();
            
            let systems_span = tracing::debug_span!("systems_execution");
            let _systems_guard = systems_span.enter();
            
            for system in self.systems.iter_mut() {
                let mut system = system.lock().unwrap();
                let name = system.name();
                let sub = system.get_subscribed_topics();

                let system_span = tracing::debug_span!("system_execution", system = name.as_str());
                let _system_guard = system_span.enter();

                let mut inputs = DataView::new();
                for topic in sub.iter() {
                    inputs = inputs
                        .add_query(&mut self.data_store.lock().unwrap(), topic)
                        .unwrap();
                }

                let new_data = system.execute(&inputs, self.dt.clone());
                self.data_store
                .lock()
                    .unwrap()
                    .apply_view(new_data)
                    .unwrap();
            }
            drop(_systems_guard);

            let end_time = Timepoint::now();
            let elapsed = end_time - start_time;

            let pubsub_span = tracing::debug_span!("pubsub_tick");
            let _pubsub_guard = pubsub_span.enter();
            if let Some(node) = &mut self.pubsub_node {
                node.tick();
            }
            drop(_pubsub_guard);

            let sleep_time = self.dt.clone().secs() - elapsed.secs();
            if sleep_time < 0.0 {
                warn!("System is taking too long to run! {:?}s", sleep_time);
            } else {
                let sleep_duration = Duration::from_secs_f64(sleep_time);

                if self.real_time {
                    let sleep_span = tracing::debug_span!("real_time_sleep", duration_ms = sleep_duration.as_millis());
                    let _sleep_guard = sleep_span.enter();
                    std::thread::sleep(sleep_duration);
                }
            }

            self.current_time = self.current_time.clone() + self.dt.clone();
            
            let sync_span = tracing::debug_span!("datastore_sync");
            let _sync_guard = sync_span.enter();
            self.data_store.lock().unwrap().run_sync();
        }

        let time_remaining_ms = ((end_time - self.current_time.clone()).secs() * 1000.0) as i64;
        tracing::event!(tracing::Level::INFO, time_remaining_ms, "main_loop_complete");
        info!("Finished running main loop");
    }
}
