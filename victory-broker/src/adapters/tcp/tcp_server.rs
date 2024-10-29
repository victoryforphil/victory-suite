use std::{
    collections::{BTreeMap, HashMap}, io::{Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread::{self, JoinHandle}
};

use log::{debug, error, info, trace};

use tracing::warn;
use victory_wtf::Timespan;

use crate::{
    adapters::{tcp::TCPPacket, PubSubAdapter},
    channel::PubSubChannelIDType,
    messages::PubSubMessage,
};
#[derive(Debug, Clone)]
pub struct TCPServerOptions {
    pub port: u16,
    pub address: String,
    pub update_interval: Timespan,
}
pub type TCPClientMapHandle = Arc<Mutex<BTreeMap<PubSubChannelIDType, TcpStream>>>;

struct ListenerAgent {
    options: TCPServerOptions,
    listener: TcpListener,
    clients_out: TCPClientMapHandle,
}

impl ListenerAgent {
    pub fn make_client_map() -> TCPClientMapHandle {
        Arc::new(Mutex::new(BTreeMap::new()))
    }
    pub fn start(options: TCPServerOptions, clients_out: TCPClientMapHandle) -> JoinHandle<()> {
        info!(
            "Starting ListenerAgent from thread: {:?} with options: {:#?}",
            std::thread::current().id(),
            options
        );
        thread::spawn( move || {
            debug!(
                "ListenerAgent new thread: {:?}",
                std::thread::current().id()
            );
            let url = format!("{}:{}", options.address, options.port);
            info!("Starting TCP server on: {}", url);
            let listener = TcpListener::bind(url);
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
                agent.tick();
                thread::sleep(options.update_interval.as_duration());
            }
        })
    }

    fn tick(&mut self) {
        let (stream, addr) = self.listener.accept().unwrap();
        debug!("New TCP Stream: {:?}", addr);
        {
            let id = rand::random::<PubSubChannelIDType>();
            self.clients_out.lock().unwrap().insert(id, stream);
            debug!("New client registered: {:?} from {:?}", id, addr);
        }
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
        format!("tcp://{}:{}", self.options.address, self.options.port)
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let clients = self.clients.lock().unwrap();
        let n_clients = clients.len();
        let mut stats = HashMap::new();
        stats.insert("n_clients".to_string(), n_clients.to_string());
        stats
    }

    fn write(&mut self, to_send: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>) {
        let clients = self.clients.clone();
        if to_send.is_empty() {
            return;
        }

        thread::spawn(move || {
            debug!(
                "Spawned new TCP write task: {:?}",
                std::thread::current().id()
            );
            for (id, messages) in to_send {
                let clients = clients.lock().unwrap();

                let mut client = match clients.first_key_value() {
                    Some(client) => client.1,
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

                trace!(
                    "Sending TCPPacket to client: {:?} with {} messages",
                    id,
                    n_messages
                );
                client.write(packet.as_slice()).unwrap();
            }
        });
    }

    fn read(&mut self) -> HashMap<PubSubChannelIDType, Vec<PubSubMessage>> {
        let clients = self.clients.clone();
        let mut res = HashMap::new();
        let mut client_write_lock = match clients.try_lock() {
            Ok(lock) => lock,
            Err(e) => {
                warn!("Failed to get write lock on clients: {:?}", e);
                return res;
            }
        };
        for (id, stream) in client_write_lock.iter_mut() {
            let mut buffer = vec![0; 1024];
            match stream.read(&mut buffer) {
                Ok(n) => {
                    buffer = buffer[..n].to_vec();
                }
                Err(e) => {
                    warn!("Failed to read from stream: {:?}", e);
                    continue;
                }
            };

            let packet: TCPPacket = match bincode::deserialize(&buffer) {
                Ok(packet) => packet,
                Err(e) => {
                    warn!("Failed to deserialize TCPPacket: {:?}", e);
                    continue;
                }
            };

            if packet.messages.is_empty() {
                continue;
            }
            let id = packet.to;
            debug!(
                "Received TCPPacket from client: {:?} with {} messages",
                id,
                packet.messages.len()
            );
            res.insert(id, packet.messages);
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
