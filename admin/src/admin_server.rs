use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{debug, info};
use prost::Message;

use tokio::sync::mpsc::Receiver;
use tonic::{transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;

use crate::{
    proto::pubsub_admin::pub_sub_admin_service_server::PubSubAdminServiceServer,
    services::pubsub::{PubSubAdmin, PubSubCommand},
};

pub struct AdminServer {}

impl AdminServer {
    pub async fn start() -> Result<Receiver<PubSubCommand>, Box<dyn std::error::Error>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let addr = "0.0.0.0:5050".parse().unwrap();
            let service = PubSubAdmin::new(tx);
            let service = PubSubAdminServiceServer::new(service);
            let service = tonic_web::enable(service);
            info!("Server listening on {}", addr);

            Server::builder()
                .accept_http1(true)
                .add_service(service)
                .serve(addr)
                .await
                .unwrap();
        });
        info!("Admin server started");
        Ok(rx)
    }
}
