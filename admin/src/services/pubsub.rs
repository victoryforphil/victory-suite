use crate::proto::pubsub_admin;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{debug, info};
use prost::Message;

use tokio::sync::{mpsc, oneshot};
use tonic::{transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;

/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = oneshot::Sender<T>;

#[derive(Debug)]
pub enum PubSubCommand {
    GetChannels {
        resp: Responder<Vec<pubsub_admin::PubSubChannel>>,
    },
}
pub struct PubSubAdmin {
    pubsub_tx: mpsc::Sender<PubSubCommand>,
}
impl PubSubAdmin {
    pub fn new(pubsub_tx: mpsc::Sender<PubSubCommand>) -> Self {
        PubSubAdmin { pubsub_tx }
    }
}
#[tonic::async_trait]
impl pubsub_admin::pub_sub_admin_service_server::PubSubAdminService for PubSubAdmin {
    async fn request_channels(
        &self,
        request: Request<pubsub_admin::ChannelRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<pubsub_admin::ChannelResponse>, Status> {
        // Return an instance of type HelloReply

        let inner = request.into_inner();
        info!("Got a request: {:?}", inner.clone());

        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = PubSubCommand::GetChannels { resp: resp_tx };

        self.pubsub_tx.send(cmd).await.unwrap();
        debug!("Sent command to get channels, waiting for response");
        let channels = resp_rx.await.unwrap();
        debug!("Got response: {:?}", channels);
        let reply = pubsub_admin::ChannelResponse { channels };
        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
