#[macro_use]
extern crate slugify;
use crate::service::LocalCommService;
use localcomm::local_comm_server::{LocalComm, LocalCommServer};
use localcomm::{HelloReply, HelloRequest};
use tokio::signal;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod service;

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
    let service = LocalCommService::new("_localcomm._tcp.local.");
    service.start();

    let addr = "0.0.0.0:50051".parse()?;
    let localcomm = LocalCommApp::default();

    println!("LocalComm instance listening on {}", addr);
    let server = Server::builder()
        .add_service(LocalCommServer::new(localcomm))
        .serve(addr);

    // This macro simply allows for cancelling all async operation as soon as one finishes.
    tokio::select! {
        result = server => {
            result?;
        }
        _ = signal::ctrl_c() => {
            println!("Ctrl+C received, shutting down...");
        }
    }

    service.stop();

    Ok(())
}
