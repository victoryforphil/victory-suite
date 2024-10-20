use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

pub mod adapters;
pub mod channel;
pub mod client;
pub mod messages;
pub mod node;
pub mod server;

pub type MutexType<T> = Arc<Mutex<T>>;
pub type RwLockType<T> = Arc<RwLock<T>>;
