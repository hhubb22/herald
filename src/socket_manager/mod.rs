use std::os::fd::AsRawFd;

use socket2::{Domain, Type};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum Error {
    #[error("Failed to create socket: {0}")]
    CreateSocketError(std::io::Error),
    #[error("Failed to set socket option: {0}")]
    SetSocketOptionError(std::io::Error),
    #[error("Invalid socket domain")]
    InvalidSocketDomain,
    #[error("Not implemented")]
    NotImplemented,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::CreateSocketError(err)
    }
}

pub struct Socket {
    pub socket: socket2::Socket,
    pub domain: Domain,
}

impl Socket {
    pub fn new() -> Result<Self, Error> {
        let domain = Domain::IPV4;
        let socket = socket2::Socket::new(domain, Type::DGRAM, None)?;
        Ok(Self { socket, domain })
    }

    #[cfg(target_os = "linux")]
    pub fn bind_to_device_v4(&self, interface: &str) -> Result<(), Error> {
        if self.domain != Domain::IPV4 {
            return Err(Error::InvalidSocketDomain);
        }
        
        unsafe {
            let ret = libc::setsockopt(
                self.socket.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_BINDTODEVICE,
                interface.as_ptr() as *const _ as *const libc::c_void,
                interface.len() as libc::socklen_t,
            );
            if ret < 0 {
                return Err(Error::SetSocketOptionError(std::io::Error::last_os_error()));
            }
        }
        
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn bind_to_device_v4(&self, interface: &str) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }
}