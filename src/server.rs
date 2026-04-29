#[macro_use]
extern crate slugify;

use crate::localcomm::{
    Device, Empty, GetDeviceListRequest, GetDeviceListResponse, RunCommandRequest, TextTypeRequest,
};
use crate::service::{LocalCommDevice, LocalCommService};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use localcomm::local_comm_server::{LocalComm, LocalCommServer};
use std::sync::{Arc, Mutex};
use tokio::signal;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod service;

pub mod localcomm {
    tonic::include_proto!("localcomm");
}

#[derive(Debug)]
pub struct LocalCommApp {
    device_list: Arc<Mutex<Vec<LocalCommDevice>>>,
}

impl LocalCommApp {
    pub fn new(device_list: Arc<Mutex<Vec<LocalCommDevice>>>) -> Self {
        LocalCommApp { device_list }
    }
}

#[tonic::async_trait]
impl LocalComm for LocalCommApp {
    async fn get_device_list(
        &self,
        request: Request<GetDeviceListRequest>,
    ) -> Result<Response<GetDeviceListResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let device_list: Vec<Device> = self
            .device_list
            .lock()
            .unwrap()
            .iter()
            .map(|d| Device {
                name: d.name.clone(),
                address: d.address.clone(),
            })
            .collect();

        Ok(Response::new(GetDeviceListResponse { list: device_list }))
    }

    async fn type_text(
        &self,
        request: Request<TextTypeRequest>,
    ) -> Result<Response<Empty>, Status> {
        let mut enigo =
            Enigo::new(&Settings::default()).map_err(|e| Status::unknown(e.to_string()))?;

        let req = request.into_inner();
        let text = req.text;

        enigo
            .text(text.as_str())
            .map_err(|e| Status::unknown(e.to_string()))
            .unwrap_or_default();

        if req.submit {
            enigo.key(Key::Return, Direction::Click).unwrap_or_default();
        }

        Ok(Response::new(Empty {}))
    }

    async fn run_command(
        &self,
        request: Request<RunCommandRequest>,
    ) -> Result<Response<Empty>, Status> {
        std::process::Command::new(request.into_inner().command).spawn()?;

        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut service = LocalCommService::new("_localcomm._tcp.local.");
    service.start();

    let addr = "0.0.0.0:50051".parse()?;
    let localcomm = LocalCommApp::new(service.devices.clone());

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
