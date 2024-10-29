use std::collections::HashMap;

use log::debug;

use crate::{channel::PubSubChannelIDType, messages::PubSubMessage};

use super::PubSubAdapter;

pub struct MockPubSubAdapter {
    read_buffer: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>,
    write_buffer: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>,
}

impl MockPubSubAdapter {
    pub fn new() -> Self {
        MockPubSubAdapter {
            read_buffer: HashMap::new(),
            write_buffer: HashMap::new(),
        }
    }
    pub fn channel_ids(&self) -> Vec<PubSubChannelIDType> {
        self.write_buffer.keys().cloned().collect()
    }
    pub fn channel_read(&mut self, channel_id: PubSubChannelIDType) -> Vec<PubSubMessage> {
        self.write_buffer
            .remove(&channel_id)
            .unwrap_or_else(Vec::new)
    }

    pub fn channel_write(&mut self, channel_id: PubSubChannelIDType, messages: Vec<PubSubMessage>) {
        self.read_buffer
            .entry(channel_id)
            .or_insert_with(Vec::new)
            .extend(messages);
    }
}

impl PubSubAdapter for MockPubSubAdapter {
    fn read(&mut self) -> HashMap<PubSubChannelIDType, Vec<PubSubMessage>> {
        // Drain read the buffer
        let mut buffer = HashMap::new();
        std::mem::swap(&mut self.read_buffer, &mut buffer);
        buffer
    }

    fn write(&mut self, to_send: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>) {
        debug!("MockPubSubAdapter::write: {:?}", to_send.len());
        for (channel_id, messages) in to_send {
            self.write_buffer
                .entry(channel_id)
                .or_insert_with(Vec::new)
                .extend(messages);
        }

        debug!("Mock Write Buffer Length {:?}", self.write_buffer.len());
    }

    fn get_name(&self) -> String {
        "MockPubSubAdapter".to_string()
    }
}
