use std::sync::{Arc, Mutex};

use datastore::topics::TopicKey;

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
    adapter
        .lock()
        .unwrap()
        .client_write(0, vec![PubSubMessage::Register()]);
    for tick in 0..10 {
        server.tick();

        match tick {
            2 => {
                adapter.lock().unwrap().client_write(
                    0,
                    vec![PubSubMessage::Subscribe(SubscribeMessage {
                        topic: TopicKey::from_str("test"),
                    })],
                );
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
