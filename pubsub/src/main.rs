use std::sync::{Arc, Mutex};

use datastore::{
    primitives::timestamp::{VicDuration, VicInstant},
    topics::TopicKey,
};

use log::info;
use pubsub::{
    adapters::{mock::MockPubSubAdapter, PubSubAdapterHandle},
    messages::*,
    server::PubSubServer,
};

mod adapters;
mod channel;
mod client;
mod messages;
fn main() {
    pretty_env_logger::init();
    let mut server = PubSubServer::new();
    let adapter = MockPubSubAdapter::new();
    let adapter = Arc::new(Mutex::new(adapter));
    server.add_adapter(adapter.clone());
    {
        adapter
            .lock()
            .unwrap()
            .client_write(0, vec![PubSubMessage::Register()]);
    }
    let start_time = VicInstant::now();
    for tick in 0..10 {
        server.tick();
        info!("Tick: {} -----------", tick);
        match tick {
            2 => {
                adapter.lock().unwrap().client_write(
                    0,
                    vec![PubSubMessage::Subscribe(SubscribeMessage {
                        topic: TopicKey::from_str("test"),
                    })],
                );
            }

            5..=10 => {
                adapter.lock().unwrap().client_write(
                    1,
                    vec![PubSubMessage::Publish(PublishMessage::primitive(
                        &TopicKey::from_str("test"),
                        Arc::new(start_time.clone() + VicDuration::new_secs(tick as f64)),
                        tick.into(),
                    ))],
                );
            }
            _ => {}
        }
    }

    // Try and read any updates
    let updates = adapter.lock().unwrap().client_read(0);
    for update in updates {
        match update {
            PubSubMessage::Update(msg) => {
                info!("Update: {:?}", msg.messages);
            }
            _ => {}
        }
    }
}
#[cfg(test)]

mod tests {
    use super::*;
    use datastore::topics::TopicKey;
    use pubsub::client::PubSubClientIDType;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_pubsub_server() {
        main();
    }
}
