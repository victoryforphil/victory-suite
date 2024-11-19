pub mod mock;
pub mod tcp;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use super::{
    packet::{SyncMessage, SyncRegisterMessage, SyncUpdateMessage},
    SyncConnectionIDType,
};

#[derive(thiserror::Error, Debug)]
pub enum AdapterError {
    #[error("Generic adapter error: {0}")]
    GenericError(String),

    #[error("Error sending message: {0}")]
    SendError(String),

    #[error("Error receiving message: {0}")]
    ReceiveError(String),
}

pub trait SyncAdapter: Debug + Send + Sync {
    fn get_name(&self) -> String;

    fn get_description(&self) -> String {
        format!("Adapter: {}", self.get_name())
    }

    fn get_stats(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn get_anon_connections(&self) -> Vec<SyncConnectionIDType>;

    fn set_register_message(&mut self, message: SyncMessage) {}

    fn get_live(&self) -> bool {
        true
    }

    fn read(&mut self) -> Result<Vec<SyncMessage>, AdapterError>;
    fn write(&mut self, to_send: Vec<SyncMessage>) -> Result<(), AdapterError>;
}
pub type SyncAdapterHandle = Arc<Mutex<dyn SyncAdapter + Send>>;

#[cfg(test)]
mod tests_sync_adapter {
    use victory_wtf::Timepoint;

    use super::*;
    use crate::{
        datapoints::Datapoint,
        primitives::Primitives,
        sync::{adapters::mock::MockSyncAdapter, SyncSubscriptionIDType},
        topics::TopicKey,
    };

    fn get_random_sub_id() -> SyncSubscriptionIDType {
        rand::random()
    }

    fn get_random_datapoints() -> Vec<Datapoint> {
        vec![Datapoint::new(
            &TopicKey::from_str("test/topic/a"),
            Timepoint::now(),
            Primitives::Text("test".to_string()),
        )]
    }

    #[test]
    fn test_mock_register_client() {
        let mut adapter = MockSyncAdapter::new();
        let msg = SyncMessage::new_register(0,vec!["test_topic".to_string()]);
        adapter.register_client(msg).unwrap();
        assert_eq!(adapter.get_new_clients().unwrap(), vec![msg]);
    }

    #[test]
    fn test_mock_read_write() {
        let mut adapter = MockSyncAdapter::new();
        let message = SyncMessage::new_update(get_random_sub_id(), get_random_datapoints());
        adapter.write(vec![message.clone()]).unwrap();

        let read_messages = adapter.get_sent_messages();
        assert_eq!(read_messages.len(), 1);
        assert_eq!(read_messages[0], message);
    }
}
