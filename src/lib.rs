//! # Herald - A Robust DHCPv4 Client
//!
//! Herald is a modern DHCPv4 client implementation written in Rust, designed for
//! reliability, performance, and ease of use. It implements the complete DORA
//! (Discover, Offer, Request, Acknowledge) process to obtain IP leases from DHCP servers.
//!
//! ## Features
//!
//! - Complete DHCPv4 protocol implementation
//! - Asynchronous operation using Tokio
//! - Robust error handling
//! - Network interface configuration
//! - Cross-platform support (Linux focus)
//!
//! ## Example
//!
//! ```rust,no_run
//! use herald::{DhcpClient, ClientConfig};
//! use bytes::Bytes;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
//!     let config = ClientConfig::new("eth0".to_string(), mac_addr);
//!     let mut client = DhcpClient::new(config).await?;
//!     let lease = client.run().await?;
//!     println!("Obtained lease: {:?}", lease);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod network;
pub mod v4;

pub use client::{DhcpClient, Lease};
pub use config::{Args, ClientConfig};
pub use error::HeraldError;
