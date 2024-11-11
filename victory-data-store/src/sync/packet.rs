use serde::{Deserialize, Serialize};

use crate::datapoints::Datapoint;

use super::{SyncConnectionIDType, SyncSubscriptionIDType};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncUpdateMessage{
    pub sub_id: SyncSubscriptionIDType,
    pub connection_id: Option<SyncConnectionIDType>,
    pub client_name: Option<String>,
    pub datapoints: Vec<Datapoint>,
}

impl SyncUpdateMessage{
    pub fn new(sub_id: SyncSubscriptionIDType, datapoints: Vec<Datapoint>) -> Self{
        Self{sub_id, client_name: None, connection_id: None, datapoints}
    }

    pub fn set_connection_id(&mut self, connection_id: SyncConnectionIDType){
        self.connection_id = Some(connection_id);
    }

    pub fn set_client_name(&mut self, client_name: String){
        self.client_name = Some(client_name);
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncRegisterMessage{
    pub sub_id: SyncSubscriptionIDType,
    pub client_name: Option<String>,
    pub connection_id: Option<SyncConnectionIDType>,
    pub subscriptions: Vec<String>,
}

impl SyncRegisterMessage{
    pub fn new(sub_id: SyncSubscriptionIDType, subscriptions: Vec<String>) -> Self{
        Self{sub_id, client_name: None, connection_id: None, subscriptions}
    }

    pub fn set_connection_id(&mut self, connection_id: SyncConnectionIDType){
        self.connection_id = Some(connection_id);
    }

    pub fn set_client_name(&mut self, client_name: String){
        self.client_name = Some(client_name);
    }
}   