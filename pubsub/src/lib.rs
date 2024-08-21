use std::sync::Arc;

pub mod adapters;
pub mod channel;
pub mod client;
pub mod messages;
pub mod node;
pub mod server;

pub type MutexType<T> = Arc<tokio::sync::Mutex<T>>;
pub type RwLockType<T> = Arc<tokio::sync::RwLock<T>>;
