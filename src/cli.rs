use crate::localcomm::local_comm_client::LocalCommClient;
use crate::localcomm::{GetDeviceListRequest, RunCommandRequest, TextTypeRequest};
use clap::{Parser, Subcommand};
use tonic::Request;
use tonic::transport::Channel;

pub mod localcomm {
    tonic::include_proto!("localcomm");
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
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
        #[arg(short, long)]
        submit: bool,
    },
    RunCommand {
        #[arg(short, long)]
        device: String,
        #[arg(short, long)]
        command: String,
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
            submit,
        }) => {
            let mut client = create_device_client(&mut client, device_name.as_str()).await;
            let request = Request::new(TextTypeRequest {
                text: text.clone(),
                submit: *submit,
            });
            client.type_text(request).await?;
        }
        Some(Commands::ListDevices) => {
            let request = Request::new(GetDeviceListRequest {});
            let response = client.get_device_list(request).await?;

            response.into_inner().list.iter().for_each(|d| {
                println!("{}: {}", d.name, d.address);
            });
        }
        Some(Commands::RunCommand { device, command }) => {
            let mut client = create_device_client(&mut client, device.as_str()).await;
            let request = Request::new(RunCommandRequest {
                command: command.to_string(),
            });
            client.run_command(request).await?;
        }
        None => {}
    };

    Ok(())
}

async fn create_device_client(
    local_client: &mut LocalCommClient<Channel>,
    device_name: &str,
) -> LocalCommClient<Channel> {
    let request = Request::new(GetDeviceListRequest {});
    let response = local_client.get_device_list(request).await.unwrap();
    let address = response
        .into_inner()
        .list
        .iter()
        .find(|d| d.name == *device_name)
        .expect("Device not found!")
        .address
        .clone();

    LocalCommClient::connect(address).await.unwrap()
}
