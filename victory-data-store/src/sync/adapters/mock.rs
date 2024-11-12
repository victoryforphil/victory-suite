use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::debug;

use super::{SyncAdapter, SyncAdapterHandle};
use crate::sync::{packet::*, SyncConnectionIDType};
#[derive(Debug)]
pub struct MockSyncAdapter {
    read_buffer: Vec<SyncMessage>,
    write_buffer: Vec<SyncMessage>,
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
    pub fn inject_messages(&mut self, messages: Vec<SyncMessage>) {
        self.read_buffer.extend(messages);
    }

    // Test helper to read messages that were sent via write_updates
    pub fn get_sent_messages(&mut self) -> Vec<SyncMessage> {
        std::mem::take(&mut self.write_buffer)
    }

    pub fn register_client(
        &mut self,
        registration: SyncMessage,
    ) -> Result<(), super::AdapterError> {
        if let SyncMessageType::Register(reg) = registration.msg {
            self.register_buffer.push(reg);
            Ok(())
        } else {
            Err(super::AdapterError::GenericError(
                "Not a register message".to_string(),
            ))
        }
    }

    pub fn get_new_clients(&mut self) -> Result<Vec<SyncRegisterMessage>, super::AdapterError> {
        Ok(std::mem::take(&mut self.register_buffer))
    }
}

impl SyncAdapter for MockSyncAdapter {
    fn get_name(&self) -> String {
        "Mock Adapter".to_string()
    }

    fn get_anon_connections(&self) -> Vec<SyncConnectionIDType> {
        vec![] // Mock has no real connections
    }

    fn read(&mut self) -> Result<Vec<SyncMessage>, super::AdapterError> {
        Ok(std::mem::take(&mut self.read_buffer))
    }

    fn write(&mut self, to_send: Vec<SyncMessage>) -> Result<(), super::AdapterError> {
        self.write_buffer.extend(to_send);
        Ok(())
    }
}

// See mod.rs for tests that use this adapter
