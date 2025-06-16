mod client;
mod config;
mod error;
mod network;
mod v4;

use crate::{
    client::DhcpClient,
    config::{Args, ClientConfig},
    error::HeraldError,
};
use bytes::BufMut as _;
use clap::Parser as _;
use tokio::fs;

async fn get_mac_address(interface: &str) -> Result<bytes::Bytes, HeraldError> {
    let path = format!("/sys/class/net/{interface}/address");
    let mac_str = fs::read_to_string(&path)
        .await
        .map_err(|_| HeraldError::InterfaceInvalid(interface.to_string()))?;

    let mut bytes = bytes::BytesMut::new();
    for byte_str in mac_str.trim().split(':') {
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| HeraldError::MacParse(mac_str.trim().to_string()))?;
        bytes.put_u8(byte);
    }
    Ok(bytes.freeze())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mac_addr = match get_mac_address(&args.interface).await {
        Ok(mac) => {
            let mac_str = mac
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(":");
            tracing::info!(
                "Found MAC address {} for interface {}",
                mac_str,
                &args.interface
            );
            mac
        }
        Err(e) => {
            tracing::error!("{}", e);
            return;
        }
    };

    let config = ClientConfig::new(args.interface, mac_addr);

    let mut client = match DhcpClient::new(config).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to initialize DHCP client: {}", e);
            return;
        }
    };

    match client.run().await {
        Ok(lease) => {
            tracing::info!("Successfully obtained lease: {:?}", lease);
        }
        Err(e) => {
            tracing::error!("DHCP client failed: {}", e);
        }
    }
}
