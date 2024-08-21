
use log::info;

use pubsub::server::PubSubServerHandle;
use tonic::transport::Server;

use crate::{
    proto::pubsub_admin::{
        pub_sub_admin_service_server::PubSubAdminServiceServer,
    },
    services::pubsub::PubSubAdmin,
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
