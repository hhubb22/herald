mod socket_manager;

use clap::Parser;
use socket_manager::{Error, Socket};
use std::error::Error as StdError;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The network interface to bind to (e.g., 'eth0', 'lo')
    interface: String,
}

fn print_error_chain(e: &Error) {
    eprintln!("Error: {}", e);
    let mut source = e.source();
    while let Some(err) = source {
        eprintln!("  Caused by: {}", err);
        source = err.source();
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Create and configure the socket using our robust factory function.
    let socket = match Socket::new_bound_to_device_and_port(&args.interface, 68) {
        Ok(socket) => {
            println!(
                "Successfully created and bound socket to interface '{}' and port 68.",
                &args.interface
            );
            socket
        }
        Err(e) => {
            print_error_chain(&e);
            std::process::exit(1);
        }
    };

    // Convert to a Tokio-compatible socket for async operations.
    let tokio_socket = match socket.to_tokio_udp_socket() {
        Ok(socket) => socket,
        Err(e) => {
            print_error_chain(&e);
            std::process::exit(1);
        }
    };

    // Send a broadcast packet.
    println!("Sending a broadcast packet to 255.255.255.255:67...");
    match tokio_socket
        .send_to(b"Hello from herald!", "255.255.255.255:67")
        .await
    {
        Ok(bytes_sent) => println!("Successfully sent {} bytes.", bytes_sent),
        Err(e) => {
            eprintln!("Failed to send packet: {}", e);
            std::process::exit(1);
        }
    }
}