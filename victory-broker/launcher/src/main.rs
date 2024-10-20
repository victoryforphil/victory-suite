use std::sync::Arc;

pub type MutexType<T> = Arc<tokio::sync::Mutex<T>>;
pub type RwLockType<T> = Arc<tokio::sync::RwLock<T>>;

#[tokio::main]
async fn main() {
    env_logger::init();
}
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_pubsub_server() {
        main();
    }
}
