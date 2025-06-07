# herald

A DHCPv4 client written in Rust, currently under development. It aims to implement the full DORA (Discover, Offer, Request, Acknowledge) process to obtain an IP lease from a DHCP server. This project demonstrates asynchronous network programming with Tokio, DHCP message construction, and low-level socket manipulation for network interface binding on Linux systems.

**Note:** This client partially implements the DHCPv4 protocol. Some features, like the completion of the Requesting/Bound states and automatic network configuration, are still in progress.

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

*   [bytes](https://crates.io/crates/bytes): For efficient byte array and buffer manipulation.
*   [clap](https://crates.io/crates/clap): For parsing command-line arguments.
*   [dhcproto](https://crates.io/crates/dhcproto): For encoding and decoding DHCP messages.
*   [libc](https://crates.io/crates/libc): Provides raw C library bindings, used here for specific socket options (`SO_BINDTODEVICE`).
*   [rand](https://crates.io/crates/rand): Used for generating random numbers, such as the DHCP transaction ID (XID).
*   [socket2](https://crates.io/crates/socket2): For advanced socket configuration options not available in the standard library.
*   [thiserror](https://crates.io/crates/thiserror): A utility for creating ergonomic custom error types.
*   [tokio](https://crates.io/crates/tokio): An asynchronous runtime for network applications, providing non-blocking I/O, timers, etc.
*   [tracing](https://crates.io/crates/tracing): A framework for instrumenting Rust programs to collect structured, event-based diagnostic information.
*   [tracing-subscriber](https://crates.io/crates/tracing-subscriber): For collecting and processing `tracing` data (e.g., printing logs to the console).

## Configuration

### Command-Line Arguments

The application accepts the following command-line arguments:

*   `-i, --interface <INTERFACE_NAME>`: **Required**. Specifies the network interface to use for DHCP communication (e.g., `eth0`, `wlan0`).

### Internal Configuration

Several parameters are currently hardcoded within the `src/config.rs` file in the `ClientConfig::new` function:

*   **Client Port:** UDP port `68` (standard for DHCP clients).
*   **Server Port:** UDP port `67` (standard for DHCP servers).
*   **Broadcast Address:** `255.255.255.255`.
*   **Initial Timeout:** `5 seconds` (e.g., for waiting for DHCPOFFER).
*   **Request Timeout:** `10 seconds` (e.g., for waiting for DHCPACK after a DHCPREQUEST).

Future versions may allow more of these internal parameters to be configured via command-line arguments or a configuration file.

## Error Handling

The `herald` client uses a custom error enum, `HeraldError` (defined in `src/error.rs`), to manage and report various issues that can occur. This includes:

*   **Socket Errors:** Problems related to network socket creation or configuration (`SocketError` from `src/network/mod.rs`).
*   **I/O Errors:** Standard input/output errors.
*   **Protocol Errors:** Issues encountered during the parsing or validation of DHCP messages.
*   **MAC Address Parsing Errors:** Failure to correctly parse a MAC address string.
*   **Invalid Interface Errors:** If the specified network interface is not found or doesn't have a MAC address.
*   **Critical Errors:** For unrecoverable situations within the state machine.

The application uses the `thiserror` crate to define these error types, providing clear and structured error information.

## Current Limitations / Future Work

`herald` is currently under development, and several features are not yet fully implemented:

*   **Incomplete DHCPv4 State Machine:**
    *   The `Requesting` state in `src/v4/handler.rs` is not fully implemented. Logic for handling DHCPACK/DHCPNAK messages and transitioning to the `Bound` state needs completion.
    *   Lease renewal, rebinding, and releasing (DHCPRELEASE) functionalities are not yet implemented.
*   **Network Interface Configuration:** The `src/network/configurator.rs` module is a placeholder. The logic to apply the obtained IP address, subnet mask, router, and DNS settings to the actual network interface is missing.
*   **Platform Specificity:** The current socket setup, specifically binding to a network device using `SO_BINDTODEVICE` in `src/network/mod.rs`, is Linux-specific. A fallback or alternative implementations for other operating systems (like macOS or Windows) are not provided.
*   **Limited Configuration:** Most client parameters (timeouts, specific DHCP options to request) are hardcoded. Future work could involve making these configurable.
*   **Testing:** More comprehensive unit and integration tests are needed.

Future development will focus on addressing these limitations to create a more robust and feature-complete DHCPv4 client.

## Project Structure

*   `Cargo.toml`: Defines project metadata, dependencies, and build settings.
*   `src/main.rs`: Contains the application entry point, command-line argument parsing, and orchestration of the DHCP client.
*   `src/client.rs`: Implements the core DHCP client logic, including the state machine and handling of network events.
*   `src/config.rs`: Defines structures for command-line arguments (`Args`) and client configuration (`ClientConfig`).
*   `src/error.rs`: Specifies custom error types (`HeraldError`) for robust error handling throughout the application.
*   `src/network/mod.rs`: Provides utilities for network socket creation and configuration, including binding to specific network devices (Linux-specific).
*   `src/network/configurator.rs`: (Currently unused) Intended for future implementation of network interface configuration after a DHCP lease is obtained.
*   `src/v4/mod.rs`: Module for DHCPv4 specific logic.
*   `src/v4/handler.rs`: Implements the DHCPv4 state machine (`DhcpV4Handler`) responsible for processing DHCP messages and managing client states (Init, Selecting, Requesting, Bound).
*   `src/v4/message.rs`: Contains functions for constructing DHCPv4 messages like Discover and Request packets.

## Functionality

The `herald` application implements a DHCPv4 client with the following core functionality:

1.  **Command-Line Interface:** Parses the network interface name (e.g., `eth0`, `wlan0`) using the `clap` crate.
2.  **MAC Address Retrieval:** Automatically fetches the MAC address of the specified network interface from `/sys/class/net/<interface_name>/address` (on Linux systems).
3.  **Socket Management (`src/network/mod.rs`):**
    *   Creates a UDP socket using Tokio.
    *   Configures the socket for broadcast messages.
    *   Binds the socket to the specified network interface and DHCP client port (68). This feature (`SO_BINDTODEVICE`) is Linux-specific.
    *   Sets the socket to non-blocking mode.
4.  **DHCPv4 State Machine (`src/v4/handler.rs` and `src/client.rs`):**
    The client progresses through several states to acquire an IP lease:
    *   **Init:** The initial state.
    *   **Selecting:**
        *   Constructs and broadcasts a DHCPDISCOVER message (`src/v4/message.rs`).
        *   Waits for DHCPOFFER messages from DHCP servers.
    *   **Requesting:** (Partially Implemented)
        *   Upon receiving a DHCPOFFER, it's planned to select an offer.
        *   Constructs and sends a DHCPREQUEST message to the chosen server, requesting the offered IP address and other parameters.
    *   **Bound:** (Partially Implemented)
        *   If a DHCPACK (Acknowledge) is received from the server, the client transitions to the Bound state.
        *   The lease information (IP address, subnet mask, router, DNS, lease duration) is stored.
        *   (Future work: The client would then configure the network interface with these details and handle lease renewal/rebinding.)
5.  **DHCP Message Handling (`dhcproto` crate):**
    *   Constructs DHCP messages (Discover, Request) with appropriate options like Client Identifier, Parameter Request List, etc.
    *   Decodes incoming DHCP messages to process server responses.
6.  **Asynchronous Operations:** Leverages Tokio for non-blocking network I/O and managing timeouts during the DHCP transaction.
