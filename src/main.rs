mod socket_manager;

use clap::Parser;
use socket_manager::Socket;

#[derive(Parser)]
struct Args {
    interface: String,
}

fn main() {
    let args = Args::parse();

    let socket = match Socket::new() {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Failed to create socket: {}", e);
            std::process::exit(1);
        }
    };

    match socket.bind_to_device_v4(&args.interface) {
        Ok(_) => println!("Socket bound to device: {}", args.interface),
        Err(e) => {
            eprintln!("Failed to bind to device: {}", e);
            std::process::exit(1);
        }
    }

    println!("Listening for packets on device: {}", args.interface);
}
