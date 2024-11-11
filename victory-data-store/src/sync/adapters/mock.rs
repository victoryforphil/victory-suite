use std::{collections::HashMap, sync::{Arc, Mutex}};

use log::debug;


use crate::sync::{packet::*, SyncConnectionIDType};

use super::{SyncAdapter, SyncAdapterHandle};
#[derive(Debug)]
pub struct MockSyncAdapter {
    read_buffer: Vec<SyncUpdateMessage>,
    write_buffer: Vec<SyncUpdateMessage>,
    register_buffer: Vec<SyncRegisterMessage>,
    my_register: Option<SyncRegisterMessage>,
}

impl MockSyncAdapter {
    pub fn new() -> Self {
        MockSyncAdapter {
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
            register_buffer: Vec::new(),
            my_register: None,
        }
    }

    pub fn as_handle(self) -> SyncAdapterHandle {
        Arc::new(Mutex::new(self))
    }

    // Test helper to directly write messages that will be returned by read_updates
    pub fn inject_messages(&mut self, messages: Vec<SyncUpdateMessage>) {
        self.read_buffer.extend(messages);
    }

    // Test helper to read messages that were sent via write_updates
    pub fn get_sent_messages(&mut self) -> Vec<SyncUpdateMessage> {
        std::mem::take(&mut self.write_buffer)
    }
}

impl SyncAdapter for MockSyncAdapter {
    fn get_name(&self) -> String {
        "Mock Adapter".to_string()
    }

    fn get_connections(&self) -> Vec<SyncConnectionIDType> {
        vec![] // Mock has no real connections
    }

    fn read_updates(&mut self) -> Result<Vec<SyncUpdateMessage>, super::AdapterError> {
        Ok(std::mem::take(&mut self.read_buffer))
    }

    fn write_updates(&mut self, to_send: Vec<SyncUpdateMessage>) -> Result<(), super::AdapterError> {
        self.write_buffer.extend(to_send);
        Ok(())
    }

    fn register_client(&mut self, registration: SyncRegisterMessage) -> Result<(), super::AdapterError> {
        self.register_buffer.push(registration);
        Ok(())
    }

    fn get_new_clients(&mut self) -> Result<Vec<SyncRegisterMessage>, super::AdapterError> {
        Ok(std::mem::take(&mut self.register_buffer))
    }
}


// See mod.rs for tests that use this adapter