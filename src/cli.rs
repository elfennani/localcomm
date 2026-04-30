use crate::localcomm::local_comm_client::LocalCommClient;
use crate::localcomm::{GetDeviceListRequest, RunCommandRequest, SendFileRequest, TextTypeRequest};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Read;
use std::path::Path;
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
    SendFile {
        #[arg(short, long)]
        device: String,
        #[arg(short, long)]
        path: String,
        #[arg(short, long)]
        buffer: Option<u8>,
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
        Some(Commands::SendFile {
            device,
            path,
            buffer,
        }) => {
            let mut client = create_device_client(&mut client, device.as_str()).await;
            let file_name = path.split("/").last().unwrap();
            let path = Path::new(path);
            let mut file = File::open(path).expect("Failed to open file");
            let mut written: u64 = 0;
            let size = std::fs::metadata(path)
                .expect("Failed to read metadata")
                .len();
            let buffer_size: usize = buffer.unwrap_or((128 * 1024) as u8) as usize;
            let progress_bar = ProgressBar::new(size)
                .with_style(
                    ProgressStyle::default_bar()
                        .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?,
                )
                .with_message(format!("Sending {}", file_name));

            if path.is_dir() {
                panic!("Path is a directory");
            }

            loop {
                let mut buffer = vec![0u8; buffer_size];
                let n = file.read(&mut buffer[..])?;

                if n == 0 {
                    break;
                }

                let request = Request::new(SendFileRequest {
                    name: file_name.to_string(),
                    position: written,
                    bytes: buffer[..n].to_vec(),
                    size,
                    buffer_size: 128 * 1024,
                });

                client.send_file(request).await?;

                written += n as u64;
                progress_bar.set_position(written)
            }

            progress_bar.finish_with_message(format!("{} sent!", file_name));
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
