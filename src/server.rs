use local_ip_address::local_ip;
use localcomm::local_comm_server::{LocalComm, LocalCommServer};
use localcomm::{HelloReply, HelloRequest};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use tokio::task;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod localcomm {
    tonic::include_proto!("localcomm");
}

#[derive(Debug, Default)]
pub struct LocalCommApp {}

#[tonic::async_trait]
impl LocalComm for LocalCommApp {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = "0.0.0.0:50051".parse()?;
    let localcomm = LocalCommApp::default();

    task::spawn(async {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");

        let receiver = mdns.monitor().expect("Failed to monitor daemon");
        task::spawn(async move {
            while let Ok(event) = receiver.recv() {
                match event {
                    mdns_sd::DaemonEvent::Error(error) => {
                        eprintln!("Daemon error: {error}");
                    }
                    event => {
                        println!("{event:?}");
                    }
                }
            }
        });

        // Create a service info.
        // Make sure that the service name: "mdns-sd-my-test" is not longer than the max length limit (15 by default).
        let service_type = "_localcomm._tcp.local.";
        let instance_name = "localcomm_instance";
        let ip = local_ip().unwrap().to_string();
        let host_name = "localcomm.local.";
        let port = 5200;
        let properties = [("property_1", "test"), ("property_2", "1234")];

        println!(
            "Broadcasting mDNS service ({}) on {}:{} with host name {}",
            service_type, ip, port, host_name
        );

        let my_service = ServiceInfo::new(
            service_type,
            instance_name,
            &host_name,
            ip,
            port,
            &properties[..],
        )
        .unwrap();

        // Register with the daemon, which publishes the service.
        mdns.register(my_service)
            .expect("Failed to register our service");
    });

    println!("LocalComm instance listening on {}", addr);
    Server::builder()
        .add_service(LocalCommServer::new(localcomm))
        .serve(addr)
        .await?;

    Ok(())
}
