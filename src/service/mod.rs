use std::rc::Rc;
use std::sync::Arc;
use local_ip_address::local_ip;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::task;

pub struct LocalCommService {
    service_type: String,
}

impl LocalCommService {
    pub(crate) fn new(service_type: &str) -> Self {
        LocalCommService {
            service_type: service_type.to_string(),
        }
    }

    pub fn start(&self) {
        self.broadcast_service();
        self.start_discovery();
    }

    fn broadcast_service(&self) {
        let service_type = self.service_type.clone();

        task::spawn(async move {
            println!("Broadcast service started");
            let mdns = ServiceDaemon::new().expect("Failed to create daemon");

            let receiver = mdns.monitor().expect("Failed to monitor daemon");
            task::spawn(async move {
                while let Ok(event) = receiver.recv() {
                    match event {
                        mdns_sd::DaemonEvent::Error(error) => {
                            eprintln!("[SERVICE_BROADCAST] Daemon error: {error}");
                        }
                        event => {
                            println!("[SERVICE_BROADCAST] {event:?}");
                        }
                    }
                }
            });

            // Create a service info.
            // Make sure that the service name: "mdns-sd-my-test" is not longer than the max length limit (15 by default).

            let instance_name = "localcomm_instance";
            let ip = local_ip().unwrap().to_string();
            let host_name = "localcomm.local.";
            let port = 5200;
            let properties = [("property_1", "test"), ("property_2", "1234")];

            println!(
                "[SERVICE_BROADCAST] Broadcasting mDNS service ({}) on {}:{} with host name {}",
                service_type, ip, port, host_name
            );

            let my_service = ServiceInfo::new(
                &service_type,
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
    }

    fn start_discovery(&self) {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");

        let service_type = self.service_type.clone();
        let receiver = mdns.browse(&service_type).expect("Failed to browse");

        task::spawn(async move {
            println!("[SERVICE_DISCOVERY] Discovery started");
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(resolved) => {
                        println!("[SERVICE_DISCOVERY] Resolved a new service: {}", resolved.fullname);
                    }
                    other_event => {
                        println!("[SERVICE_DISCOVERY] Received other event: {:?}", &other_event);
                    }
                }
            }
        });
    }
}
