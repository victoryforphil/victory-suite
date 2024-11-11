use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SyncConfig{
    pub client_name: String,
    pub subscriptions: Vec<String>,
}