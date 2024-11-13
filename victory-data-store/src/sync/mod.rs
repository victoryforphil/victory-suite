use std::sync::{Arc, Mutex};

use adapters::SyncAdapter;
use config::SyncConfig;
use log::{debug, info};
use packet::{SyncMessage, SyncMessageType, SyncRegisterMessage, SyncUpdateMessage};
use subscription::Subscription;

use crate::{
    database::{listener::DataStoreListener, Datastore, DatastoreHandle},
    datapoints::Datapoint,
    topics::{self, TopicKey},
};

pub mod adapters;
pub mod config;
pub mod packet;
pub mod subscription;
pub type SyncConnectionIDType = u16;
pub type SyncSubscriptionIDType = u16;
pub type DatastoreSyncHandle = Arc<Mutex<DatastoreSync>>;
pub type SyncAdapterHandle = Arc<Mutex<dyn SyncAdapter>>;

/// A syncronization object for a datastore
/// Will manage a pool of adapters and track incoming / new connections
/// from said adapters.
///
/// Will route incoming messages and update the linked datastore (handle)
///
/// Will route outgoing messages to the appropriate adapters using the connection ID

#[derive(Debug)]
pub struct DatastoreSync {
    pub config: SyncConfig,
    adapter: SyncAdapterHandle,
    /// Represents requested subscriptions from the remote datastore
    /// that we need to send updates to
    remote_subscriptions: Vec<Subscription>,
    /// Represents all subscriptions that we locally want to receive updates for
    /// from the remote datastore
    local_subscriptions: Vec<Subscription>,

    new_datapoints: Vec<Datapoint>,
}

impl DatastoreSync {
    pub fn new(config: SyncConfig, adapter: SyncAdapterHandle) -> Self {
        debug!(
            "[Sync/DatastoreSync] New datastore sync with config: {:#?}",
            config
        );
        let mut new_sync = Self {
            config: config.clone(),
            adapter,
            remote_subscriptions: Vec::new(),
            local_subscriptions: Vec::new(),
            new_datapoints: Vec::new(),
        };
        new_sync.subscribe_from_strings(config.subscriptions.clone());
        new_sync
    }
    #[tracing::instrument(skip_all)]
    pub fn as_handle(self) -> DatastoreSyncHandle {
        Arc::new(Mutex::new(self))
    }

    /// Given a topic filter, have this datastore subscribe to all topics that match the filter
    /// from remote datastores
    #[tracing::instrument(skip_all)]
    pub fn subscribe(&mut self, topic_filter: TopicKey) {
        debug!(
            "[Sync/Subscribe] Subscribing to updates for: {:?}",
            topic_filter
        );
        let new_subscription = Subscription::new(self.config.client_name.clone(), topic_filter);
        self.send_register_subscriptions(&new_subscription);
        self.local_subscriptions.push(new_subscription);
    }

    /// Given an array of topic strings, subscribe to all topics that match the filter
    #[tracing::instrument(skip_all)]
    pub fn subscribe_from_strings(&mut self, topic_filters: Vec<String>) {
        for topic_filter in topic_filters {
            self.subscribe(TopicKey::from_str(&topic_filter));
        }
    }

    /// Called externally to handle a new remote subscription registration
    /// This should be called once a new remote client has subscribed to a channel
    #[tracing::instrument(skip_all)]
    pub fn remote_subscribe(&mut self, subscription: Subscription) {
        debug!(
            "[Sync/RemoteSubscribe] New remote subscription: {:?}",
            subscription
        );
        self.remote_subscriptions.push(subscription);

        // Send all current datapoints to the new subscriber
    }
    /// Returns new subscriptions that were added
    #[tracing::instrument(skip_all)]
    pub fn sync(&mut self) -> Vec<Subscription> {
        // Get any anon connections and send them a welcome message
        let anon_connections = self.adapter.lock().unwrap().get_anon_connections();
        let local_subscriptions = self.local_subscriptions.clone();
        let mut new_subscriptions = Vec::new();
        for connection_id in anon_connections {
            for local_sub in local_subscriptions.iter() {
                info!(
                    "[Sync/Sync] Re-sending register message to new connection: {:?}",
                    connection_id
                );
                self.send_register_subscriptions(&local_sub);
            }
        }

        let mut datapoints = Vec::new();
        let mut update_count = 0;
        let mut read_result = self.adapter.lock().unwrap().read();
        while let Ok(messages) = &read_result {
            if messages.is_empty() {
                break;
            }
            for message in messages {
                match message.msg.clone() {
                    SyncMessageType::Register(register) => {
                        let new_sub = Subscription::from_register(message);
                        self.remote_subscribe(new_sub.clone());
                        new_subscriptions.push(new_sub);
                    }
                    SyncMessageType::Update(update) => {
                        update_count += update.datapoints.len();
                        datapoints.extend(update.datapoints);
                    }
                    _ => {}
                }
            }
            read_result = self.adapter.lock().unwrap().read();
        }
        if update_count > 0 {
            debug!("[Sync/Sync] Got {} new datapoints", update_count);
            self.on_remote_datapoints(datapoints);
        }
        self.send_queued_datapoints();
        new_subscriptions
    }

    /// Called externally to handle new local datapoints updates that should be sent out to any
    /// subscriptions that match the datapoint.
    #[tracing::instrument(skip_all)]
    pub fn on_local_datapoints(&mut self, datapoints: Vec<Datapoint>) {
        // For each datapoint, find all subscriptions that match and send the update
        for datapoint in datapoints {
            let matching_subscriptions: Vec<&mut Subscription> = self
                .remote_subscriptions
                .iter_mut()
                .filter(|sub| sub.is_match(&datapoint))
                .collect();
            for sub in matching_subscriptions {
                sub.push(datapoint.clone());
            }
        }
    }

    /// Called once a new remote datapoint has been received and should be stored in the local datastore
    #[tracing::instrument(skip_all)]
    pub fn on_remote_datapoints(&mut self, datapoints: Vec<Datapoint>) {
        self.new_datapoints.extend(datapoints);
    }

    #[tracing::instrument(skip_all)]
    pub fn drain_new_datapoints(&mut self) -> Vec<Datapoint> {
        self.new_datapoints.drain(..).collect()
    }
}

impl DatastoreSync {
    #[tracing::instrument(skip_all)]
    fn send_register_subscriptions(&mut self, subscription: &Subscription) {
        let mut message = SyncMessage::new_register(
            subscription.sub_id,
            vec![subscription.topic_query.display_name()],
        );
        message.client_name = Some(self.config.client_name.clone());
        debug!("[Sync/Register] Sending register message: {:?}", message);

        self.adapter.lock().unwrap().write(vec![message]).unwrap();
    }

    #[tracing::instrument(skip_all)]
    fn send_pure_register(&mut self) {
        let message = SyncMessage::new_register(0, vec![]);

        debug!(
            "[Sync/Register] Sending pure register message: {:?}",
            message
        );

        self.adapter.lock().unwrap().write(vec![message]).unwrap();
    }

    #[tracing::instrument(skip_all)]
    fn send_queued_datapoints(&mut self) {
        for sub in self.remote_subscriptions.iter_mut() {
            if sub.len() > 0 {
                let mut datapoints = Vec::new();
                while let Some(datapoint) = sub.pop() {
                    datapoints.push(datapoint);
                }

                for batch in datapoints.chunks(8) {
                    let message = SyncMessage::new_update(sub.sub_id, batch.to_vec());
                    self.adapter.lock().unwrap().write(vec![message]).unwrap();
                }
            }
        }
    }
}

impl DataStoreListener for DatastoreSync {
    #[tracing::instrument(skip_all)]
    fn on_datapoint(&mut self, datapoint: &crate::datapoints::Datapoint) {
        self.on_local_datapoints(vec![datapoint.clone()]);
    }

    #[tracing::instrument(skip_all)]
    fn on_bucket_update(&mut self, bucket: &crate::buckets::BucketHandle) {}
}

#[cfg(test)]
mod tests {
    use adapters::mock::MockSyncAdapter;

    use super::*;

    #[test]
    fn test_datastore_setup_sync() {
        env_logger::init();
        let mut datastore = Datastore::new().handle();
        let mut mock_adapter = MockSyncAdapter::new().as_handle();

        let mut sync_config = SyncConfig {
            client_name: "test_client".to_string(),
            subscriptions: vec!["test_topic".to_string()],
        };

        datastore
            .lock()
            .unwrap()
            .setup_sync(sync_config, mock_adapter.clone());

        // Check that the adapter recieved a register message
        let register_message = mock_adapter.lock().unwrap().read().unwrap();
        assert_eq!(register_message.len(), 1);
        assert_eq!(
            register_message[0].client_name,
            Some("test_client".to_string())
        );
        assert_eq!(
            register_message[0].try_as_register().unwrap().subscriptions,
            vec!["test_topic".to_string()]
        );
    }

    #[test]
    fn test_datastore_read_new_clients() {
        env_logger::init();
        let mut datastore = Datastore::new().handle();
        let mut mock_adapter = MockSyncAdapter::new().as_handle();

        let mut sync_config = SyncConfig {
            client_name: "test_client".to_string(),
            subscriptions: vec!["test_topic".to_string()],
        };

        datastore
            .lock()
            .unwrap()
            .setup_sync(sync_config, mock_adapter.clone());

        datastore.lock().unwrap().run_sync();

        let sync = datastore.lock().unwrap().sync.clone().unwrap();
        let sync = sync.lock().unwrap();
        assert_eq!(sync.remote_subscriptions.len(), 1);
        assert_eq!(
            sync.remote_subscriptions[0].client_name,
            "test_client".to_string()
        );
        assert_eq!(
            sync.remote_subscriptions[0].topic_query.display_name(),
            "test_topic"
        );
    }
}
