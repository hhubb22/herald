#![allow(dead_code)]

use bytes::{BufMut as _, Bytes, BytesMut};
use dhcproto::{
    v4::{self, OptionCode},
    Encodable as _, Encoder,
};
use std::error::Error as StdError;

/// Constructs a DHCP Discover message.
pub fn build_dhcp_discover(mac_addr: &Bytes, xid: u32) -> Result<Vec<u8>, Box<dyn StdError>> {
    let mut msg = v4::Message::default();
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
    msg.opts_mut()
        .insert(v4::DhcpOption::ParameterRequestList(vec![
            OptionCode::SubnetMask,       // 1
            OptionCode::Router,           // 3
            OptionCode::DomainNameServer, // 6
            OptionCode::DomainName,       // 15
        ]));

    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(&mut buffer);
    msg.encode(&mut encoder)?;
    Ok(buffer)
}

/// Constructs a DHCP Request message.
pub fn build_dhcp_request(
    mac_addr: &Bytes,
    xid: u32,
    offered_ip: std::net::Ipv4Addr,
    server_ip: std::net::Ipv4Addr,
) -> Result<Vec<u8>, Box<dyn StdError>> {
    let mut msg = v4::Message::default();
    msg.set_opcode(v4::Opcode::BootRequest)
        .set_chaddr(mac_addr)
        .set_htype(v4::HType::Eth)
        .set_xid(xid)
        .set_ciaddr(std::net::Ipv4Addr::UNSPECIFIED); // Client IP, 0.0.0.0 as it's not confirmed

    // DHCP Message Type - REQUEST (3)
    msg.opts_mut()
        .insert(v4::DhcpOption::MessageType(v4::MessageType::Request));

    // Requested IP Address (Option 50)
    msg.opts_mut()
        .insert(v4::DhcpOption::RequestedIpAddress(offered_ip));

    // Server Identifier (Option 54)
    msg.opts_mut()
        .insert(v4::DhcpOption::ServerIdentifier(server_ip));

    // Client Identifier (Option 61) - same as Discover
    let mut client_id_data = BytesMut::new();
    client_id_data.put_u8(1); // htype Ethernet
    client_id_data.extend_from_slice(mac_addr);
    msg.opts_mut().insert(v4::DhcpOption::ClientIdentifier(
        client_id_data.freeze().to_vec(),
    ));

    // Parameter Request List (Option 55) - can be same as Discover
    msg.opts_mut()
        .insert(v4::DhcpOption::ParameterRequestList(vec![
            OptionCode::SubnetMask,
            OptionCode::Router,
            OptionCode::DomainNameServer,
            OptionCode::DomainName,
        ]));

    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(&mut buffer);
    msg.encode(&mut encoder)?;
    Ok(buffer)
}