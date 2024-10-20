use std::collections::HashMap;

use log::debug;

use crate::{client::PubSubClientIDType, messages::PubSubMessage};

use super::PubSubAdapter;

pub struct MockPubSubAdapter {
    read_buffer: HashMap<PubSubClientIDType, Vec<PubSubMessage>>,
    write_buffer: HashMap<PubSubClientIDType, Vec<PubSubMessage>>,
}

impl MockPubSubAdapter {
    pub fn new() -> Self {
        MockPubSubAdapter {
            read_buffer: HashMap::new(),
            write_buffer: HashMap::new(),
        }
    }
    pub fn client_ids(&self) -> Vec<PubSubClientIDType> {
        self.write_buffer.keys().cloned().collect()
    }
    pub fn client_read(&mut self, client_id: PubSubClientIDType) -> Vec<PubSubMessage> {
        self.write_buffer
            .remove(&client_id)
            .unwrap_or_else(Vec::new)
    }

    pub fn client_write(&mut self, client_id: PubSubClientIDType, messages: Vec<PubSubMessage>) {
        self.read_buffer
            .entry(client_id)
            .or_insert_with(Vec::new)
            .extend(messages);
    }
}

impl PubSubAdapter for MockPubSubAdapter {
    fn read(&mut self) -> HashMap<PubSubClientIDType, Vec<PubSubMessage>> {
        // Drain read the buffer
        let mut buffer = HashMap::new();
        std::mem::swap(&mut self.read_buffer, &mut buffer);
        buffer
    }

    fn write(&mut self, to_send: HashMap<PubSubClientIDType, Vec<PubSubMessage>>) {
        debug!("MockPubSubAdapter::write: {:?}", to_send.len());
        for (client_id, messages) in to_send {
            self.write_buffer
                .entry(client_id)
                .or_insert_with(Vec::new)
                .extend(messages);
        }

        debug!("Mock Write Buffer Length {:?}", self.write_buffer.len());
    }

    fn get_name(&self) -> String {
        "MockPubSubAdapter".to_string()
    }
}
