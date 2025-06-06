mod socket_manager;

use bytes::{BufMut, Bytes, BytesMut};
use clap::Parser;
use dhcproto::{v4, Encodable, Encoder, v4::OptionCode};
use rand::Rng as _;
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
    let xid = rand::rng().random_range(0..=u32::MAX);
    msg.set_opcode(v4::Opcode::BootRequest)
        .set_chaddr(mac_addr)
        .set_htype(v4::HType::Eth) // Ethernet
        .set_hops(0)
        .set_xid(xid) // Transaction ID
        .set_secs(0)
        .set_flags(v4::Flags::default().set_broadcast());

        // Add DHCP Message Type Option (53) - DHCPDISCOVER (1)
        msg.opts_mut()
        .insert(v4::DhcpOption::MessageType(v4::MessageType::Discover));

    // Add Client Identifier Option (61)
    // Using htype 1 (Ethernet) followed by the MAC address
    let mut client_id_data = BytesMut::new();
    client_id_data.put_u8(1); // htype Ethernet
    client_id_data.extend_from_slice(mac_addr);
    msg.opts_mut().insert(v4::DhcpOption::ClientIdentifier(
        client_id_data.freeze().to_vec(),
    ));

    // Add Parameter Request List Option (55)
    msg.opts_mut().insert(v4::DhcpOption::ParameterRequestList(vec![
        OptionCode::SubnetMask,         // 1
        OptionCode::Router,             // 3
        OptionCode::DomainNameServer,   // 6
        OptionCode::DomainName,         // 15
    ]));


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