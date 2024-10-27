use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;

pub mod adapters;
pub mod channel;
pub mod messages;
pub mod node;

pub type MutexType<T> = Arc<Mutex<T>>;
pub type RwLockType<T> = Arc<RwLock<T>>;
