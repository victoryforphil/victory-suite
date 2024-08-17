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

mod adapters;
mod channel;
mod client;
mod messages;

async fn server_main(
    server: RwLockType<PubSubServer>,
    mut admin_rx: tokio::sync::mpsc::Receiver<admin::services::pubsub::PubSubCommand>,
) {
    loop {
        info!("Server tick");
        while let Some(cmd) = admin_rx.recv().await {
            debug!("--------Received command: {:?}", cmd);
            match cmd {
                admin::services::pubsub::PubSubCommand::GetChannels { resp } => {
                    let server = server.read().await;
                    let channels = server
                        .channels
                        .iter()
                        .map(|(topic, chan)| pubsub_admin::PubSubChannel {
                            topic: topic.to_string(),
                            subscribers: vec![],
                            publishers: vec![],
                            message_count: chan.try_lock().unwrap().update_queue.len() as i32,
                        })
                        .collect();
                    resp.send(channels).unwrap();
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
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
    }
    let server = Arc::new(tokio::sync::RwLock::new(server));

    let admin_rx = AdminServer::start().await.unwrap();
    info!("Admin server started");
    let manager_server_handle = server.clone();

    tokio::spawn(async move {
        info!("Starting server main");
        server_main(manager_server_handle, admin_rx).await;
    });

    let main_tick = tokio::spawn(async move {
        // Main tick thread that runs at 1hz
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
                            &TopicKey::from_str("test"),
                            Arc::new(start_time.clone() + VicDuration::new_secs(tick_count as f64)),
                            tick_count.into(),
                        ))],
                    );
                }
                _ => {}
            }
        }
    });

    main_tick.await.unwrap();
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
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
