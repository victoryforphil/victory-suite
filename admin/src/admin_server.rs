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
    services::pubsub::{PubSubAdmin, PubSubCommand},
};

pub struct AdminServer {}

impl AdminServer {
    pub async fn start(server: Arc<Mutex<PubSubServer>>) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let server_handle = tokio::spawn(async move {
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

        let manager_handle = tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                debug!("Received command: {:?}", cmd);
                match cmd {
                    PubSubCommand::GetChannels { resp } => {
                        loop {
                            {
                                let server = server.lock().unwrap();
                                let channels = server
                                    .channels
                                    .iter()
                                    .map(|(topic, chan)| pubsub_admin::PubSubChannel {
                                        topic: topic.to_string(),
                                        subscribers: vec![],
                                        publishers: vec![],
                                        message_count: chan.try_lock().unwrap().update_queue.len()
                                            as i32,
                                    })
                                    .collect();
                                resp.send(Ok(ChannelResponse { channels }));
                            }
                            // SLeep for 1 second
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }
                };
            }
        });

        Ok(())
    }
}
