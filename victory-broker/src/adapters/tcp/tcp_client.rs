use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use log::{debug, info, trace, warn};

use crate::{
    adapters::{tcp::TCPPacket, PubSubAdapter},
    channel::PubSubChannelIDType,
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
    id: Option<PubSubChannelIDType>,
    buffer: Vec<u8>,
}

impl TCPClientAdapter {
    pub fn new(options: TCPClientOptions) -> Result<TCPClientAdapter, Box<dyn std::error::Error>> {
        let url = options.to_url();
        info!("Connecting to: {}", url);
        let stream = TcpStream::connect(url)?;
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(TCPClientAdapter {
            options,
            stream: Arc::new(Mutex::new(stream)),
            id: None,
            buffer: vec![],
        })
    }
}

impl PubSubAdapter for TCPClientAdapter {
    fn read(&mut self) -> HashMap<PubSubChannelIDType, Vec<PubSubMessage>> {
        let mut res = HashMap::new();
        let mut stream = self.stream.try_lock().unwrap();
        let packet: TCPPacket = match bincode::deserialize_from(&mut *stream) {
            Ok(packet) => packet,
            Err(e) => {
                return res;
            }
        };
        let id = packet.to;
        trace!(
            "Received TCPPacket from client: {:?} with {} messages",
            id.to_string(),
            packet.messages.len()
        );
        res.insert(id, packet.messages);
        res
    }

    fn write(&mut self, to_send: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>) {
        for (id, messages) in to_send.into_iter() {
            let mut stream = self.stream.try_lock().unwrap();
            let packet = TCPPacket {
                from: id,
                to: id,
                messages,
            };

            match bincode::serialize_into(&mut *stream, &packet) {
                Ok(_) => (),
                Err(e) => {
                    warn!("Failed to serialize / send TCPPacket: {:?}", e);
                    return;
                }
            };
        }
    }

    fn get_name(&self) -> String {
        "TCPClientAdapter".to_string()
    }
}
