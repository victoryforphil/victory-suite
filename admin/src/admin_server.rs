use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{debug, info};
use prost::Message;

use pubsub::server::{PubSubServer, PubSubServerHandle};
use tokio::sync::mpsc::Receiver;
use tonic::{
    transport::{Channel, Server},
    Request, Response, Status,
};
use tonic_web::GrpcWebLayer;

use crate::{
    proto::pubsub_admin::{
        self, pub_sub_admin_service_server::PubSubAdminServiceServer, ChannelResponse,
    },
    services::pubsub::{PubSubAdmin},
};

pub struct AdminServer {}

impl AdminServer {
    pub async fn start(server: PubSubServerHandle) -> Result<(), Box<dyn std::error::Error>> {
       

        let server_handle = tokio::spawn(async move {
            let addr = "0.0.0.0:5050".parse().unwrap();
            let service = PubSubAdmin::new(server);
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

     

        Ok(())
    }
}
