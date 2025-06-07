use std::{io, net::UdpSocket as StdUdpSocket};
use thiserror::Error;
use tokio::net::UdpSocket as TokioUdpSocket;

/// Defines all possible errors for socket operations.
#[derive(Error, Debug)]
pub enum SocketError {
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

    #[error("Failed to set SO_REUSEADDR on socket")]
    SetReuseAddress(#[source] io::Error),

    #[error("Failed to set socket to non-blocking mode")]
    SetNonBlocking(#[source] io::Error),

    #[error("Failed to convert socket to TokioUdpSocket")]
    ConvertToTokio(#[source] io::Error),

    #[allow(dead_code)]
    #[error("Binding to a specific device is not implemented on this platform")]
    NotImplemented,
}

/// Creates a new `tokio::net::UdpSocket` bound to a specific network device and port.
///
/// This function handles the low-level socket configuration required for operations
/// like sending broadcast packets from a specific interface.
///
/// # Arguments
/// * `interface` - The name of the network interface (e.g., "eth0").
/// * `port` - The port number to bind the socket to.
///
/// # Returns
/// A `Result` containing the configured `TokioUdpSocket` or a `SocketError`.
#[cfg(target_os = "linux")]
pub fn new_tokio_socket_bound_to_device(
    interface: &str,
    port: u16,
) -> Result<TokioUdpSocket, SocketError> {
    use socket2::{Domain, Socket, Type};
    use std::os::fd::AsRawFd;

    // Create a socket2 socket, which allows setting options before binding.
    let socket2 =
        Socket::new(Domain::IPV4, Type::DGRAM, None).map_err(SocketError::CreateSocket)?;

    // Set `SO_BROADCAST`. This is required for sending broadcast messages.
    socket2
        .set_broadcast(true)
        .map_err(SocketError::SetBroadcast)?;

    // Set `SO_REUSEADDR`. Allows binding to an address that is already in use.
    socket2
        .set_reuse_address(true)
        .map_err(SocketError::SetReuseAddress)?;

    // Set `SO_BINDTODEVICE`. This is an unsafe raw syscall.
    // It is safe here because we use a valid file descriptor and correct parameters.
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
        return Err(SocketError::BindToDevice {
            interface: interface.to_string(),
            source: io::Error::last_os_error(),
        });
    }

    // Bind the socket to the address and port.
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    socket2.bind(&addr.into()).map_err(SocketError::BindSocket)?;

    // Convert to a standard socket, then into a Tokio socket.
    let std_socket: StdUdpSocket = socket2.into();
    std_socket
        .set_nonblocking(true)
        .map_err(SocketError::SetNonBlocking)?;
    TokioUdpSocket::from_std(std_socket).map_err(SocketError::ConvertToTokio)
}

/// Fallback for non-Linux systems where `SO_BINDTODEVICE` is not available.
#[cfg(not(target_os = "linux"))]
pub fn new_tokio_socket_bound_to_device(
    _interface: &str,
    _port: u16,
) -> Result<TokioUdpSocket, SocketError> {
    Err(SocketError::NotImplemented)
}