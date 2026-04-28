use crate::localcomm::HelloRequest;
use crate::localcomm::local_comm_client::LocalCommClient;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
    /// Simply prints a name
    PrintName {
        name: String,
        #[arg(long)]
        age: i8
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        Some(Commands::PrintName { name, age }) => {
            println!("Your name is {} ({}yo)", name, age);
        }
        None => {}
    }

    let mut client = LocalCommClient::connect("http://localcomm.local:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Nizar".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={}", response.into_inner().message);

    Ok(())
}