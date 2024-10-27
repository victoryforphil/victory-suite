use std::{collections::HashMap, sync::Arc};

use log::{debug, info, warn};
use tokio::{net::TcpStream, sync::Mutex};

use crate::{
    adapters::{tcp::TCPPacket, PubSubAdapter},
    client::PubSubClientIDType,
    messages::PubSubMessage,
};

#[derive(Debug, Clone)]
pub struct TCPClientOptions {
    pub port: u16,
    pub address: String,
}

impl TCPClientOptions {
    pub fn new(port: u16, address: String) -> TCPClientOptions {
        TCPClientOptions { port, address }
    }

    pub fn from_url(url: &str) -> TCPClientOptions {
        let parts: Vec<&str> = url.split(":").collect();
        let port = parts[1].parse::<u16>().unwrap();
        let address = parts[0].to_string();
        TCPClientOptions { port, address }
    }

    pub fn to_url(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

pub struct TCPClientAdapter {
    options: TCPClientOptions,
    stream: Arc<Mutex<TcpStream>>,
    id: Option<PubSubClientIDType>,
}

impl TCPClientAdapter {
    pub async fn new(options: TCPClientOptions) -> TCPClientAdapter {
        let url = options.to_url();
        info!("Connecting to: {}", url);
        let stream = TcpStream::connect(url).await.unwrap();

        TCPClientAdapter {
            options,
            stream: Arc::new(Mutex::new(stream)),
            id: None,
        }
    }
}

impl PubSubAdapter for TCPClientAdapter {
    fn read(&mut self) -> HashMap<PubSubClientIDType, Vec<PubSubMessage>> {
        let stream = self.stream.clone();
        let id = self.id.unwrap_or(404);
        let mut res = HashMap::new();
        let mut buffer = vec![0; 1024];
        let stream = stream.try_lock().unwrap();
        match stream.try_read(&mut buffer) {
            Ok(n) => {
                buffer = buffer[..n].to_vec();
            }
            Err(e) => {
                //   warn!("Failed to read from stream: {:?}", e);
                return res;
            }
        };

        let packet: TCPPacket = bincode::deserialize(&buffer).unwrap();
        info!(
            "Received TCPPacket from client: {:?} with {} messages",
            id.to_string(),
            packet.messages.len()
        );
        res.insert(id, packet.messages);
        res
    }

    fn write(&mut self, to_send: HashMap<PubSubClientIDType, Vec<PubSubMessage>>) {
        let stream = self.stream.clone();
        let id = self.id.unwrap_or(404);

        tokio::spawn(async move {
            let stream = stream.lock().await;
            for (_, messages) in to_send.iter() {
                let n_messages = messages.len();
                let packet = TCPPacket {
                    from: id,
                    to: 0,
                    messages: messages.clone(),
                };
                let packet = bincode::serialize(&packet).unwrap();
                let size = packet.len() as u32;
                debug!(
                    "Sending TCPPacket of size {} bytes, containing {} messages",
                    size, n_messages
                );
                stream.try_write(packet.as_slice()).unwrap();
            }
        });
    }

    fn get_name(&self) -> String {
        "TCPClientAdapter".to_string()
    }
}
