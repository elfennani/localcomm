use crate::localcomm::local_comm_client::LocalCommClient;
use crate::localcomm::{GetDeviceListRequest, HelloRequest, TextTypeRequest};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tonic::Request;

pub mod localcomm {
    tonic::include_proto!("localcomm");
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ListDevices,
    Type {
        #[arg(short, long)]
        text: String,
        #[arg(short, long)]
        device: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut client = LocalCommClient::connect("http://localhost:50051").await?;

    match &cli.command {
        Some(Commands::Type {
            text,
            device: device_name,
        }) => {
            let request = Request::new(GetDeviceListRequest {});
            let response = client.get_device_list(request).await?;
            let address = response
                .into_inner()
                .list
                .iter()
                .find(|d| d.name == *device_name)
                .expect("Device not found!")
                .address
                .clone();

            let mut client = LocalCommClient::connect(address).await?;
            let request = Request::new(TextTypeRequest { text: text.clone() });
            client.type_text(request).await?;
        }
        Some(Commands::ListDevices) => {
            let request = Request::new(GetDeviceListRequest {});
            let response = client.get_device_list(request).await?;

            response.into_inner().list.iter().for_each(|d| {
                println!("{}: {}", d.name, d.address);
            });
        }
        _ => {}
    };

    Ok(())
}
