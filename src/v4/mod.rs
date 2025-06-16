//! DHCPv4 protocol implementation
//!
//! This module contains the DHCPv4-specific implementation including:
//! - Message construction and parsing
//! - State machine handling
//! - Protocol-specific logic

pub mod handler;
pub mod message;

pub use handler::DhcpV4Handler;
pub use message::{build_dhcp_discover, build_dhcp_request};
