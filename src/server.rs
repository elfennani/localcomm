use tonic::{Request, Response, Status};
use tonic::transport::Server;
use localcomm::{HelloReply, HelloRequest};
use localcomm::local_comm_server::{LocalComm, LocalCommServer};

pub mod localcomm {
    tonic::include_proto!("localcomm");
}

#[derive(Debug, Default)]
pub struct LocalCommApp {}

#[tonic::async_trait]
impl LocalComm for LocalCommApp {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> { // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let localcomm = LocalCommApp::default();

    println!("LocalComm instance listening on {}", addr);
    Server::builder()
        .add_service(LocalCommServer::new(localcomm))
        .serve(addr)
        .await?;

    Ok(())
}