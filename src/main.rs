mod socket_manager;

use bytes::{BufMut, Bytes, BytesMut};
use clap::Parser;
use dhcproto::{v4, Encodable, Encoder};
use std::error::Error as StdError;
use std::num::ParseIntError;
use tokio::fs;

const DHCP_CLIENT_PORT: u16 = 68;
const DHCP_SERVER_PORT: u16 = 67;
const BROADCAST_ADDRESS: &str = "255.255.255.255";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The network interface to bind to (e.g., 'eth0', 'lo')
    #[arg(short, long)]
    interface: String,
}

/// Parses a MAC address string (e.g., "0a:1b:2c:3d:4e:5f") into a `Bytes` object.
fn parse_mac_address(mac_str: &str) -> Result<Bytes, ParseIntError> {
    let mut bytes = BytesMut::new();
    for byte_str in mac_str.split(':') {
        if !byte_str.is_empty() {
            let byte = u8::from_str_radix(byte_str, 16)?;
            bytes.put_u8(byte);
        }
    }
    Ok(bytes.freeze())
}

/// Constructs a DHCP Discover message.
fn build_dhcp_discover(mac_addr: &Bytes) -> Result<Vec<u8>, Box<dyn StdError>> {
    let mut msg = v4::Message::default();
    msg.set_opcode(v4::Opcode::BootRequest)
        .set_chaddr(mac_addr)
        .set_htype(v4::HType::Eth) // Ethernet
        .set_hops(0)
        .set_xid(0x12345678) // Transaction ID
        .set_secs(0)
        .set_flags(v4::Flags::default().set_broadcast());

    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(&mut buffer);
    msg.encode(&mut encoder)?;
    Ok(buffer)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let args = Args::parse();

    println!(
        "Attempting to bind to interface '{}' and port {}...",
        &args.interface, DHCP_CLIENT_PORT
    );

    // Create and configure the socket using our robust factory function.
    let socket = socket_manager::new_tokio_socket_bound_to_device(
        &args.interface,
        DHCP_CLIENT_PORT,
    )?;

    println!("Socket created and bound successfully.");

    // Read the hardware (MAC) address from the system.
    let mac_path = format!("/sys/class/net/{}/address", &args.interface);
    let mac_str = fs::read_to_string(&mac_path).await?;
    let mac_addr = parse_mac_address(mac_str.trim())?;

    println!("Found MAC address: {}", mac_str.trim());

    // Build the DHCP Discover packet.
    let dhcp_packet = build_dhcp_discover(&mac_addr)?;

    println!("Constructed DHCP Discover packet. Broadcasting...");

    // Send the packet to the broadcast address.
    let target = format!("{}:{}", BROADCAST_ADDRESS, DHCP_SERVER_PORT);
    let bytes_sent = socket.send_to(&dhcp_packet, &target).await?;

    println!(
        "Successfully sent {} bytes to {}.",
        bytes_sent, target
    );

    Ok(())
}