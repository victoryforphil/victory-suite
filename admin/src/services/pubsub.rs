use crate::proto::pubsub_admin::{self, ChannelRequest, ChannelResponse, PubSubChannel};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use log::{debug, info};
use prost::Message;

use pubsub::server::{PubSubServer, PubSubServerHandle};
use std::pin::Pin;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;
/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.

pub struct PubSubAdmin {
    server: PubSubServerHandle,
}
impl PubSubAdmin {
    pub fn new(server: PubSubServerHandle) -> Self {
        PubSubAdmin { server }
    }
}
#[tonic::async_trait]
impl pubsub_admin::pub_sub_admin_service_server::PubSubAdminService for PubSubAdmin {
    type RequestChannelsStream = ReceiverStream<Result<ChannelResponse, Status>>;
    async fn request_channels(
        &self,
        request: Request<pubsub_admin::ChannelRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<Self::RequestChannelsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(12);

        println!("EchoServer::server_streaming_echo");
        let server = self.server.clone();
  
        tokio::spawn(async move {
            loop {
                let channels = server.read().await.channels.clone();
               
                let channel_data = channels
                    .iter()
                    .map(|(topic, clients)| {
                        let clients = clients.try_lock().unwrap();
                        return PubSubChannel {
                        topic: topic.to_string(),
                        subscribers: clients.subscribers.iter().map(|client| client.try_lock().unwrap().id.to_string()).collect(),
                        publishers: clients.publishers.iter().map(|client| client.try_lock().unwrap().id.to_string()).collect(),
                        message_count: clients.get_queue_size() as i32
                    }})
                    .collect();

                let response = ChannelResponse { channels:channel_data };
                match tx.send(Ok(response)).await {
                    Ok(_) => {
                        info!("\titem sent to client");
                        // item (server response) was queued to be send to client
                    }
                    Err(_item) => {
                        info!("\tclient disconnected");
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            println!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(output_stream))
    }
}
