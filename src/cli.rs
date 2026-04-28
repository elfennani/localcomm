use crate::localcomm::HelloRequest;
use crate::localcomm::local_comm_client::LocalCommClient;

pub mod localcomm {
    tonic::include_proto!("localcomm");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = LocalCommClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Nizar".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={}", response.into_inner().message);

    Ok(())
}