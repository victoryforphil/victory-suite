pub mod mock;
pub mod tonic;

use std::{collections::HashMap, fmt::Debug, sync::{Arc, Mutex}};

use super::{
    packet::{SyncRegisterMessage, SyncUpdateMessage},
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

pub trait SyncAdapter: Debug + Send + Sync{
    fn get_name(&self) -> String;

    fn get_description(&self) -> String {
        format!("Adapter: {}", self.get_name())
    }

    fn get_stats(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn get_connections(&self) -> Vec<SyncConnectionIDType>;

    fn get_live(&self) -> bool {
        true
    }

    fn read_updates(&mut self) -> Result<Vec<SyncUpdateMessage>, AdapterError>;
    fn write_updates(&mut self, to_send: Vec<SyncUpdateMessage>) -> Result<(), AdapterError>;

    fn register_client(&mut self, registration: SyncRegisterMessage) -> Result<(), AdapterError>;

    fn get_new_clients(&mut self) -> Result<Vec<SyncRegisterMessage>, AdapterError>;
}
pub type SyncAdapterHandle = Arc<Mutex<dyn SyncAdapter + Send>>;

#[cfg(test)]
mod tests_sync_adapter {
    use victory_wtf::Timepoint;

    use super::*;
    use crate::{
        datapoints::Datapoint, primitives::Primitives, sync::{adapters::mock::MockSyncAdapter, SyncSubscriptionIDType},
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
        let registration =
            SyncRegisterMessage::new(get_random_sub_id(), vec!["test_topic".to_string()]);
        adapter.register_client(registration.clone()).unwrap();
        assert_eq!(adapter.get_new_clients().unwrap(), vec![registration.clone()]);
    }

    #[test]
    fn test_mock_read_write() {
        let mut adapter = MockSyncAdapter::new();
        let message = SyncUpdateMessage::new(get_random_sub_id(), get_random_datapoints());
        adapter.write_updates(vec![message.clone()]).unwrap();

        let read_messages = adapter.get_sent_messages();
        assert_eq!(read_messages.len(), 1);
        assert_eq!(read_messages[0], message);
    }
}
