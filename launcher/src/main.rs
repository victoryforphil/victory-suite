use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use admin::{admin_server::AdminServer, proto::pubsub_admin};
use datastore::{
    primitives::timestamp::{VicDuration, VicInstant},
    topics::TopicKey,
};
pub type MutexType<T> = Arc<tokio::sync::Mutex<T>>;
pub type RwLockType<T> = Arc<tokio::sync::RwLock<T>>;

use log::{debug, info};
use pubsub::{adapters::mock::MockPubSubAdapter, messages::*, server::PubSubServer};

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting pubsub server");
    let mut server = PubSubServer::new();
    let adapter = MockPubSubAdapter::new();
    let adapter = Arc::new(tokio::sync::Mutex::new(adapter));
    server.add_adapter(adapter.clone());
    {
        info!("Registering adapter");
        adapter
            .lock()
            .await
            .client_write(0, vec![PubSubMessage::Register()]);

        info!("Registering adapter");
        adapter
            .lock()
            .await
            .client_write(1, vec![PubSubMessage::Register()]);
    }
    let server = Arc::new(tokio::sync::RwLock::new(server));

    AdminServer::start(server.clone()).await.unwrap();

    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut tick_count = 0;
    loop {
        info!("[Main] Tick");
        interval.tick().await;
        let mut server = server.write().await;
        server.tick();
        tick_count += 1;
        let start_time = VicInstant::now();
        info!("[Main] Tick count: {}", tick_count);
        match tick_count {
            2 => {
                adapter.lock().await.client_write(
                    0,
                    vec![PubSubMessage::Subscribe(SubscribeMessage {
                        topic: TopicKey::from_str("test"),
                    })],
                );
            }

            5..=10 => {
                adapter.lock().await.client_write(
                    1,
                    vec![PubSubMessage::Publish(PublishMessage::primitive(
                        &TopicKey::from_str(format!("test{}", tick_count).as_str()),
                        Arc::new(start_time.clone() + VicDuration::new_secs(tick_count as f64)),
                        tick_count.into(),
                    ))],
                );
            }

            12..=500 => {
                for i in 0..10 {
                    adapter.lock().await.client_write(
                        1,
                        vec![PubSubMessage::Publish(PublishMessage::primitive(
                            &TopicKey::from_str("test"),
                            Arc::new(
                                start_time.clone() + VicDuration::new_secs((tick_count + i) as f64),
                            ),
                            tick_count.into(),
                        ))],
                    );
                }
            }

            _ => {}
        }
    }
}
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_pubsub_server() {
        main();
    }
}
