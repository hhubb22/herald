use std::io;
use std::net::UdpSocket as StdUdpSocket;
use thiserror::Error;
use tokio::net::UdpSocket as TokioUdpSocket;

/// Defines all possible errors for socket operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create a new socket")]
    CreateSocket(#[source] io::Error),

    #[error("Failed to enable broadcast on socket")]
    SetBroadcast(#[source] io::Error),

    #[error("Failed to set SO_BINDTODEVICE on interface '{interface}'")]
    BindToDevice {
        interface: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to bind socket to address")]
    BindSocket(#[source] io::Error),

    #[error("Failed to set socket to non-blocking mode")]
    SetNonBlocking(#[source] io::Error),

    #[error("Failed to convert socket to TokioUdpSocket")]
    ConvertToTokio(#[source] io::Error),

    #[allow(dead_code)]
    #[error("This operation is not implemented on the current platform")]
    NotImplemented,
}

/// A wrapper around a standard UDP socket.
pub struct Socket {
    pub socket: StdUdpSocket,
}

impl Socket {
    /// Creates a new socket, binds it to a specific network device and port.
    ///
    /// This function correctly handles the order of operations:
    /// 1. Creates a raw `socket2` socket.
    /// 2. Sets the `SO_BROADCAST` option.
    /// 3. Sets the `SO_BINDTODEVICE` option (on Linux).
    /// 4. Binds the socket to `0.0.0.0` and the specified port.
    ///
    /// This factory function is the preferred way to create a `Socket`.
    #[cfg(target_os = "linux")]
    pub fn new_bound_to_device_and_port(interface: &str, port: u16) -> Result<Self, Error> {
        use socket2::{Domain, Socket as Socket2, Type};
        use std::os::fd::AsRawFd;

        // Create a socket2 socket, which allows setting options before binding.
        let socket2 = Socket2::new(Domain::IPV4, Type::DGRAM, None)
            .map_err(Error::CreateSocket)?;


        socket2.set_broadcast(true).map_err(Error::SetBroadcast)?;

        // Set `SO_BINDTODEVICE`. This is unsafe as it's a raw syscall.
        // It's safe here because we use a valid FD and correct parameters.
        let ret = unsafe {
            libc::setsockopt(
                socket2.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_BINDTODEVICE,
                interface.as_ptr() as *const libc::c_void,
                interface.len() as libc::socklen_t,
            )
        };
        if ret < 0 {
            // Capture the OS error for better diagnostics.
            return Err(Error::BindToDevice {
                interface: interface.to_string(),
                source: io::Error::last_os_error(),
            });
        }

        // Bind the socket to the address and port.
        let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
        socket2.bind(&addr.into()).map_err(Error::BindSocket)?;

        // Convert to a standard socket and wrap it in our struct.
        Ok(Self {
            socket: std::net::UdpSocket::from(socket2),
        })
    }

    /// Fallback function for non-Linux systems where `SO_BINDTODEVICE` is not available.
    #[cfg(not(target_os = "linux"))]
    pub fn new_bound_to_device_and_port(_interface: &str, _port: u16) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    /// Consumes the `Socket` and converts it into a `tokio::net::UdpSocket`.
    pub fn to_tokio_udp_socket(self) -> Result<TokioUdpSocket, Error> {
        self.socket
            .set_nonblocking(true)
            .map_err(Error::SetNonBlocking)?;

        tokio::net::UdpSocket::from_std(self.socket)
            .map_err(Error::ConvertToTokio)
    }
}