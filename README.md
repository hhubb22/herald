# herald

A simple DHCP client that sends a DHCP Discover message to the network.

## How to Run

1.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    cd herald
    ```
2.  **Build the project:**
    ```bash
    cargo build
    ```
3.  **Run the executable:**
    The program requires the network interface to be specified using the `-i` or `--interface` flag.
    ```bash
    ./target/debug/herald -i <interface_name>
    ```
    For example, to use the `eth0` interface:
    ```bash
    ./target/debug/herald -i eth0
    ```
    **Note:** You might need to run the program with `sudo` depending on your system's network permissions.
    ```bash
    sudo ./target/debug/herald -i eth0
    ```

## Dependencies

This project uses the following key Rust crates:

*   [clap](https://crates.io/crates/clap): For parsing command-line arguments.
*   [tokio](https://crates.io/crates/tokio): For asynchronous runtime and operations.
*   [dhcproto](https://crates.io/crates/dhcproto): For constructing DHCP messages.
*   [bytes](https://crates.io/crates/bytes): For working with byte streams.
*   [socket2](https://crates.io/crates/socket2): For low-level socket configuration.
*   [thiserror](https://crates.io/crates/thiserror): For convenient error handling.

## Project Structure

*   `Cargo.toml`: Defines project metadata, dependencies, and build settings.
*   `src/main.rs`: Contains the main application logic, including command-line argument parsing, DHCP message construction, and network interaction.
*   `src/socket_manager/mod.rs`: Provides utility functions for creating and configuring network sockets.

## Functionality

The `herald` application performs the following steps:

1.  **Parses Command-Line Arguments:** It uses the `clap` crate to parse the network interface name (e.g., `eth0`, `wlan0`) provided by the user via the `-i` or `--interface` flag.
2.  **Socket Creation and Binding:** It creates a UDP socket and binds it to the specified network interface on DHCP client port 68. This is handled by the `socket_manager` module, which ensures the socket is configured correctly for network broadcast and device binding.
3.  **MAC Address Retrieval:** It reads the MAC (Media Access Control) address of the specified network interface from the `/sys/class/net/<interface_name>/address` file on Linux systems.
4.  **DHCP Discover Message Construction:** It constructs a DHCP Discover message using the `dhcproto` crate. The message includes:
    *   Opcode: BootRequest
    *   Client MAC address (CHADDR)
    *   Hardware type (HTYPE): Ethernet
    *   Transaction ID (XID)
    *   Broadcast flag
5.  **Broadcast DHCP Message:** It sends the DHCP Discover message as a UDP broadcast to the DHCP server port 67 (typically `255.255.255.255:67`).
