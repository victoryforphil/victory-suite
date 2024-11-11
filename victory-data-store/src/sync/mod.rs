use std::sync::{Arc, Mutex};

use adapters::SyncAdapter;
use config::SyncConfig;
use log::debug;
use packet::SyncRegisterMessage;
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
        };
        new_sync.subscribe_from_strings(config.subscriptions.clone());
        new_sync
    }

    pub fn as_handle(self) -> DatastoreSyncHandle {
        Arc::new(Mutex::new(self))
    }

    /// Given a topic filter, have this datastore subscribe to all topics that match the filter
    /// from remote datastores
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
    pub fn subscribe_from_strings(&mut self, topic_filters: Vec<String>) {
        for topic_filter in topic_filters {
            self.subscribe(TopicKey::from_str(&topic_filter));
        }
    }

    /// Called externally to handle a new remote subscription registration
    /// This should be called once a new remote client has subscribed to a channel
    pub fn remote_subscribe(&mut self, subscription: Subscription) {
        self.remote_subscriptions.push(subscription);
    }

    /// Called externally to handle new local datapoints updates that should be sent out to any
    /// subscriptions that match the datapoint.
    pub fn on_local_datapoints(&mut self, datapoints: Vec<Datapoint>) {}

    /// Called once a new remote datapoint has been received and should be stored in the local datastore
    pub fn on_remote_datapoints(&mut self, datapoints: Vec<Datapoint>) {}
}

impl DatastoreSync {
    fn send_register_subscriptions(&mut self, subscription: &Subscription) {
        let mut message = SyncRegisterMessage::new(subscription.sub_id, vec![subscription.topic_query.display_name()]);
        message.client_name = Some(self.config.client_name.clone());
        debug!("[Sync/Register] Sending register message: {:?}", message);
        self.adapter.lock().unwrap().register_client(message);
    }
}

impl DataStoreListener for DatastoreSync {
    fn on_datapoint(&mut self, datapoint: &crate::datapoints::Datapoint) {
        self.on_local_datapoints(vec![datapoint.clone()]);
    }

    fn on_bucket_update(&mut self, bucket: &crate::buckets::BucketHandle) {}
}


#[cfg(test)]
mod tests {
    use adapters::mock::MockSyncAdapter;

    use super::*;

    #[test]
    fn test_datastore_setup_sync(){
        env_logger::init();
        let mut datastore = Datastore::new().handle();
        let mut mock_adapter = MockSyncAdapter::new().as_handle();

        let mut sync_config = SyncConfig{
            client_name: "test_client".to_string(),
            subscriptions: vec!["test_topic".to_string()],
        };

        datastore.lock().unwrap().setup_sync(sync_config, mock_adapter.clone());

        // Check that the adapter recieved a register message
        let register_message = mock_adapter.lock().unwrap().get_new_clients();
        assert!(register_message.is_ok());
        let register_message = register_message.unwrap();
        assert_eq!(register_message.len(), 1);
        assert_eq!(register_message[0].client_name, Some("test_client".to_string()));
        assert_eq!(register_message[0].subscriptions, vec!["test_topic".to_string()]);
    }
}
