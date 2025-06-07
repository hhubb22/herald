use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The network interface to bind to (e.g., 'eth0', 'lo')
    #[arg(short, long)]
    pub interface: String,
}

#[allow(dead_code)]
pub struct ClientConfig {
    pub interface: String,
    pub mac_address: bytes::Bytes,
    pub client_port: u16,
    pub server_port: u16,
    pub broadcast_address: std::net::Ipv4Addr,
    pub initial_timeout: Duration,
    pub request_timeout: Duration,
}

impl ClientConfig {
    pub fn new(interface: String, mac_address: bytes::Bytes) -> Self {
        Self {
            interface,
            mac_address,
            client_port: 68,
            server_port: 67,
            broadcast_address: "255.255.255.255".parse().unwrap(),
            initial_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
        }
    }
}