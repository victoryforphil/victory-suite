use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use log::{debug, error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use tracing::warn;
use victory_time_rs::Timespan;

use crate::{
    adapters::{tcp::TCPPacket, PubSubAdapter},
    client::PubSubClientIDType,
    messages::PubSubMessage,
};
#[derive(Debug, Clone)]
pub struct TCPServerOptions {
    pub port: u16,
    pub address: String,
    pub update_interval: Timespan,
}
pub type TCPClientMapHandle = Arc<RwLock<BTreeMap<PubSubClientIDType, TcpStream>>>;

struct ListenerAgent {
    options: TCPServerOptions,
    listener: TcpListener,
    clients_out: TCPClientMapHandle,
}

impl ListenerAgent {
    pub fn make_client_map() -> TCPClientMapHandle {
        Arc::new(RwLock::new(BTreeMap::new()))
    }
    pub fn start(options: TCPServerOptions, clients_out: TCPClientMapHandle) -> JoinHandle<()> {
        info!(
            "Starting ListenerAgent from thread: {:?} with options: {:#?}",
            std::thread::current().id(),
            options
        );
        tokio::spawn(async move {
            debug!(
                "ListenerAgent new thread: {:?}",
                std::thread::current().id()
            );
            let url = format!("{}:{}", options.address, options.port);
            info!("Starting TCP server on: {}", url);
            let listener = TcpListener::bind(url).await;
            let listener = match listener {
                Ok(listener) => listener,
                Err(e) => {
                    error!("Failed to bind to address: {}", e);
                    return;
                }
            };

            let mut agent = ListenerAgent {
                options: options.clone(),
                listener,
                clients_out,
            };
            loop {
                agent.tick().await;
                tokio::time::sleep(options.update_interval.as_duration()).await;
            }
        })
    }

    async fn tick(&mut self) {
        let (stream, addr) = self.listener.accept().await.unwrap();
        debug!("New TCP Stream: {:?}", addr);
        let id = rand::random::<PubSubClientIDType>();
        self.clients_out.write().await.insert(id, stream);
        info!("New client registered: {:?} from {:?}", id, addr);
    }
}

type ListenerAgentHandle = Arc<Mutex<ListenerAgent>>;

pub struct TCPServerAdapter {
    clients: TCPClientMapHandle,
    agent: JoinHandle<()>,
    options: TCPServerOptions,


}

impl TCPServerAdapter {
    pub fn new(options: TCPServerOptions) -> TCPServerAdapter {
        let clients = ListenerAgent::make_client_map();
        let agent = ListenerAgent::start(options.clone(), clients.clone());
        info!("TCPServerAdapter started with options: {:#?}", options);
        TCPServerAdapter {
            agent,
            clients,
            options,
        }
    }
}

impl PubSubAdapter for TCPServerAdapter {

    fn get_description(&self) -> String {
        format!(
            "tcp://{}:{}",
            self.options.address, self.options.port
        )
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let clients = self.clients.try_read().unwrap();
        let n_clients = clients.len();
        let mut stats = HashMap::new();
        stats.insert("n_clients".to_string(), n_clients.to_string());
        stats
    }


    fn write(&mut self, to_send: HashMap<PubSubClientIDType, Vec<PubSubMessage>>) {
        let clients = self.clients.clone();

        tokio::spawn(async move {
            debug!(
                "Spawned new TCP write task: {:?}",
                std::thread::current().id()
            );
            for (id, messages) in to_send {
                let mut clients = clients.write().await;
                let client = match clients.get_mut(&id) {
                    Some(client) => client,
                    None => {
                        warn!("TCP Stream not found for client id: {:?}", id);
                        continue;
                    }
                };
                let n_messages = messages.len();
                let packet = TCPPacket {
                    from: 0,
                    to: id,
                    messages,
                };
                let packet = bincode::serialize(&packet).unwrap();
                let size = packet.len() as u32;
                debug!(
                    "Sending TCPPacket of size {} bytes, containing {} messages",
                    size, n_messages
                );
                client.try_write(packet.as_slice()).unwrap();
            }
        });
    }

    fn read(&mut self) -> HashMap<PubSubClientIDType, Vec<PubSubMessage>> {
        let clients = self.clients.clone();
        let mut res = HashMap::new();
        for (id, stream) in clients.try_write().unwrap().iter_mut() {
            let mut buffer = vec![0; 1024];
            match stream.try_read(&mut buffer) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to read from stream: {:?}", e);
                    continue;
                }
            };

            let packet: TCPPacket = bincode::deserialize(&buffer).unwrap();
            debug!(
                "Received TCPPacket from client: {:?} with {} messages",
                id,
                packet.messages.len()
            );
            res.insert(*id, packet.messages);
        }
        res
    }

    fn get_name(&self) -> String {
        "TCPServerAdapter".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_server() {
        pretty_env_logger::init();
        let options = TCPServerOptions {
            port: 8080,
            address: "0.0.0.0".to_string(),
            update_interval: Timespan::new_hz(50.0),
        };
        let adapter = TCPServerAdapter::new(options);
        assert_eq!(adapter.options.port, 8080);
    }
}
