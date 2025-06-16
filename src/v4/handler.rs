//! DHCPv4 state machine implementation
//!
//! This module implements the DHCPv4 client state machine that handles
//! the complete DORA (Discover, Offer, Request, Acknowledge) process.

use super::message::build_dhcp_discover;
use crate::{
    client::{Action, DhcpStateMachine, Event},
    error::HeraldError,
};
use bytes::Bytes;
use dhcproto::{v4, Decodable};
use std::{net::SocketAddr, str::FromStr, time::Duration};

#[derive(Debug, PartialEq, Clone, Copy)]
enum DhcpV4State {
    Init,
    Selecting,
    Requesting,
    Bound,
}

pub struct DhcpV4Handler {
    state: DhcpV4State,
    mac_address: Bytes,
    xid: u32,
    offer: Option<v4::Message>,
}

impl DhcpV4Handler {
    pub fn new(mac_address: Bytes) -> Self {
        Self {
            state: DhcpV4State::Init,
            mac_address,
            xid: rand::random(),
            offer: None,
        }
    }

    // 私有辅助函数来处理特定的状态转换
    fn handle_init(&mut self) -> Result<Action, HeraldError> {
        self.state = DhcpV4State::Selecting;
        let discover_packet = build_dhcp_discover(&self.mac_address, self.xid)?;
        let broadcast_addr = SocketAddr::from_str("255.255.255.255:67")
            .map_err(|e| HeraldError::Critical(format!("Invalid broadcast address: {e}")))?;
        Ok(Action::Send(discover_packet, broadcast_addr))
    }

    fn handle_selecting(&mut self, event: Event) -> Result<Action, HeraldError> {
        match event {
            Event::PacketReceived(data) => {
                tracing::debug!("Received packet in Selecting state, length: {}", data.len());
                let msg = v4::Message::decode(&mut v4::Decoder::new(data)).map_err(|e| {
                    tracing::error!("Failed to decode DHCP message: {}", e);
                    HeraldError::Protocol(Box::new(e))
                })?;

                tracing::debug!(
                    "Decoded message: XID={:x}, our XID={:x}",
                    msg.xid(),
                    self.xid
                );

                if msg.xid() == self.xid {
                    tracing::debug!("XID matches, checking message type");
                    if let Some(msg_type_opt) = msg.opts().get(v4::OptionCode::MessageType) {
                        tracing::debug!("Message type option found: {:?}", msg_type_opt);
                        if let v4::DhcpOption::MessageType(v4::MessageType::Offer) = msg_type_opt {
                            tracing::info!(
                                "Received DHCP OFFER from server, offered IP: {}",
                                msg.yiaddr()
                            );

                            // Check if server identifier is present
                            if let Some(v4::DhcpOption::ServerIdentifier(server_ip)) =
                                msg.opts().get(v4::OptionCode::ServerIdentifier)
                            {
                                tracing::info!("Server identifier: {}", server_ip);
                            }

                            self.offer = Some(msg);
                            self.state = DhcpV4State::Requesting;
                            tracing::info!("Transitioning to Requesting state");
                            return self.handle_requesting();
                        } else {
                            tracing::debug!("Not a DHCP OFFER message: {:?}", msg_type_opt);
                        }
                    } else {
                        tracing::debug!("No message type option found");
                    }
                } else {
                    tracing::debug!("XID mismatch, ignoring packet");
                }
                // 不是我们想要的包，继续等待
                Ok(Action::Wait(Duration::from_secs(5)))
            }
            Event::Timeout => {
                tracing::warn!("Timeout in Selecting state, retrying discovery");
                // 超时，重新发送 Discover
                self.state = DhcpV4State::Init;
                self.handle_init()
            }
        }
    }

    fn handle_requesting(&mut self) -> Result<Action, HeraldError> {
        if let Some(ref offer) = self.offer {
            // Extract server identifier and offered IP from the offer
            let server_id = offer
                .opts()
                .get(v4::OptionCode::ServerIdentifier)
                .and_then(|opt| {
                    if let v4::DhcpOption::ServerIdentifier(ip) = opt {
                        Some(*ip)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    HeraldError::Protocol(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "No server identifier in offer",
                    )))
                })?;

            let offered_ip = offer.yiaddr();

            // Build and send DHCP Request
            let request_packet = super::message::build_dhcp_request(
                &self.mac_address,
                self.xid,
                offered_ip,
                server_id,
            )
            .map_err(HeraldError::Protocol)?;

            let broadcast_addr = SocketAddr::from_str("255.255.255.255:67")
                .map_err(|e| HeraldError::Critical(format!("Invalid broadcast address: {e}")))?;
            Ok(Action::Send(request_packet, broadcast_addr))
        } else {
            Err(HeraldError::Critical(
                "No offer available for request".to_string(),
            ))
        }
    }

    fn handle_requesting_response(&mut self, event: Event) -> Result<Action, HeraldError> {
        match event {
            Event::PacketReceived(data) => {
                let msg = v4::Message::decode(&mut v4::Decoder::new(data))
                    .map_err(|e| HeraldError::Protocol(Box::new(e)))?;

                if msg.xid() == self.xid {
                    match msg.opts().get(v4::OptionCode::MessageType) {
                        Some(v4::DhcpOption::MessageType(v4::MessageType::Ack)) => {
                            // DHCP ACK received - extract lease information
                            let lease = self.extract_lease_info(&msg)?;
                            self.state = DhcpV4State::Bound;
                            Ok(Action::StoreLease(lease))
                        }
                        Some(v4::DhcpOption::MessageType(v4::MessageType::Nak)) => {
                            // DHCP NAK received - restart the process
                            tracing::warn!("Received DHCP NAK, restarting discovery");
                            self.state = DhcpV4State::Init;
                            self.offer = None;
                            self.xid = rand::random(); // New transaction ID
                            self.handle_init()
                        }
                        _ => {
                            // Not the message we're looking for, keep waiting
                            Ok(Action::Wait(Duration::from_secs(5)))
                        }
                    }
                } else {
                    // Wrong transaction ID, keep waiting
                    Ok(Action::Wait(Duration::from_secs(5)))
                }
            }
            Event::Timeout => {
                // Timeout waiting for ACK/NAK, retry request
                tracing::warn!("Timeout waiting for DHCP ACK, retrying request");
                self.handle_requesting()
            }
        }
    }

    fn extract_lease_info(&self, msg: &v4::Message) -> Result<crate::client::Lease, HeraldError> {
        let offered_ip = msg.yiaddr();

        let subnet_mask = msg.opts().get(v4::OptionCode::SubnetMask).and_then(|opt| {
            if let v4::DhcpOption::SubnetMask(mask) = opt {
                Some(*mask)
            } else {
                None
            }
        });

        let routers = msg.opts().get(v4::OptionCode::Router).and_then(|opt| {
            if let v4::DhcpOption::Router(routers) = opt {
                Some(routers.clone())
            } else {
                None
            }
        });

        let dns_servers = msg
            .opts()
            .get(v4::OptionCode::DomainNameServer)
            .and_then(|opt| {
                if let v4::DhcpOption::DomainNameServer(dns) = opt {
                    Some(dns.clone())
                } else {
                    None
                }
            });

        let lease_duration = msg
            .opts()
            .get(v4::OptionCode::AddressLeaseTime)
            .and_then(|opt| {
                if let v4::DhcpOption::AddressLeaseTime(secs) = opt {
                    Some(Duration::from_secs(*secs as u64))
                } else {
                    None
                }
            });

        let server_identifier = msg
            .opts()
            .get(v4::OptionCode::ServerIdentifier)
            .and_then(|opt| {
                if let v4::DhcpOption::ServerIdentifier(ip) = opt {
                    Some(*ip)
                } else {
                    None
                }
            });

        Ok(crate::client::Lease {
            offered_ip,
            subnet_mask,
            routers,
            dns_servers,
            lease_duration,
            server_identifier,
        })
    }
}

impl DhcpStateMachine for DhcpV4Handler {
    fn state_name(&self) -> &'static str {
        match self.state {
            DhcpV4State::Init => "Init",
            DhcpV4State::Selecting => "Selecting",
            DhcpV4State::Requesting => "Requesting",
            DhcpV4State::Bound => "Bound",
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<Action, HeraldError> {
        tracing::debug!("Handling event {:?} in state {:?}", event, self.state);
        match self.state {
            DhcpV4State::Init => self.handle_init(),
            DhcpV4State::Selecting => self.handle_selecting(event),
            DhcpV4State::Requesting => self.handle_requesting_response(event),
            DhcpV4State::Bound => {
                // In bound state, we could handle lease renewal, but for now just stay bound
                tracing::info!("Client is in Bound state - lease is active");
                Ok(Action::Exit)
            }
        }
    }
}
