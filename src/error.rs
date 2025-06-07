use crate::network::SocketError;
use std::{error::Error as StdError, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeraldError {
    #[error("Socket operation failed")]
    Socket(#[from] SocketError),

    #[error("I/O error")]
    Io(#[from] io::Error),

    #[error("DHCP protocol error")]
    Protocol(#[from] Box<dyn StdError>),

    #[error("Failed to parse MAC address: {0}")]
    MacParse(String),

    #[error("Interface '{0}' not found or has no MAC address")]
    InterfaceInvalid(String),

    #[error("State machine reached a critical failure: {0}")]
    Critical(String),
}