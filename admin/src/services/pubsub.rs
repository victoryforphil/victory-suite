use crate::proto::pubsub_admin::{self, ChannelResponse};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{debug, info};
use prost::Message;

use tokio::sync::{mpsc, oneshot};
use tonic::{transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;
use std::pin::Pin;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
/// Provided by the requester and used by the manager task to send
/// the command response back to the requester.
type Responder<T> = mpsc::Sender<T>;
type ChannelResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<ChannelResponse, Status>> + Send>>;

#[derive(Debug)]
pub enum PubSubCommand {
    GetChannels {
        resp: Responder<Result<ChannelResponse, Status>>,
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
    
    type RequestChannelsStream = ResponseStream;

    async fn request_channels(
        &self,
        request: Request<pubsub_admin::ChannelRequest>, // Accept request of type HelloRequest
    ) ->ChannelResult<Self::RequestChannelsStream> {
        let (tx, rx) = mpsc::channel(100);
        let cmd = PubSubCommand::GetChannels { resp: tx };
        self.pubsub_tx.send(cmd).await.unwrap();
       
       let output_stream = ReceiverStream::new(rx);
       Ok(Response::new(
        Box::pin(output_stream) as Self::RequestChannelsStream
    ))
    }
}
