use serde::{Deserialize, Serialize};

use crate::datapoints::Datapoint;

use super::{SyncConnectionIDType, SyncSubscriptionIDType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncMessageType {
    Register(SyncRegisterMessage),
    Update(SyncUpdateMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncMessage {
    pub sub_id: SyncSubscriptionIDType,
    pub connection_id: Option<SyncConnectionIDType>,
    pub client_name: Option<String>,
    pub msg: SyncMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncUpdateMessage {
    pub datapoints: Vec<Datapoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncRegisterMessage {
    pub subscriptions: Vec<String>,
}

impl SyncUpdateMessage {
    pub fn new(datapoints: Vec<Datapoint>) -> Self {
        Self { datapoints }
    }
}

impl SyncRegisterMessage {
    pub fn new(subscriptions: Vec<String>) -> Self {
        Self { subscriptions }
    }
}

impl SyncMessage {
    pub fn new_update(sub_id: SyncSubscriptionIDType, datapoints: Vec<Datapoint>) -> Self {
        Self {
            sub_id,
            connection_id: None,
            client_name: None,
            msg: SyncMessageType::Update(SyncUpdateMessage { datapoints }),
        }
    }

    pub fn new_register(sub_id: SyncSubscriptionIDType, subscriptions: Vec<String>) -> Self {
        Self {
            sub_id,
            connection_id: None,
            client_name: None,
            msg: SyncMessageType::Register(SyncRegisterMessage { subscriptions }),
        }
    }

    pub fn set_connection_id(&mut self, connection_id: SyncConnectionIDType) {
        self.connection_id = Some(connection_id);
    }

    pub fn set_client_name(&mut self, client_name: String) {}

    pub fn try_as_update(&self) -> Result<&SyncUpdateMessage, &'static str> {
        if let SyncMessageType::Update(update) = &self.msg {
            Ok(update)
        } else {
            Err("Message is not an update")
        }
    }

    pub fn try_as_register(&self) -> Result<&SyncRegisterMessage, &'static str> {
        if let SyncMessageType::Register(register) = &self.msg {
            Ok(register)
        } else {
            Err("Message is not a register")
        }
    }
}
