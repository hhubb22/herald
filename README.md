# Herald - A Robust DHCPv4 Client

[![CI](https://github.com/your-org/herald/workflows/CI/badge.svg)](https://github.com/your-org/herald/actions)
[![codecov](https://codecov.io/gh/your-org/herald/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/herald)
[![Crates.io](https://img.shields.io/crates/v/herald.svg)](https://crates.io/crates/herald)
[![Documentation](https://docs.rs/herald/badge.svg)](https://docs.rs/herald)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/your-org/herald#license)

A modern, robust DHCPv4 client implementation written in Rust. Herald implements the complete DORA (Discover, Offer, Request, Acknowledge) process to obtain IP leases from DHCP servers with a focus on reliability, performance, and ease of use.

## ✨ Features

- **Complete DHCPv4 Implementation**: Full DORA process with proper state machine
- **Asynchronous Operation**: Built on Tokio for high-performance networking
- **Robust Error Handling**: Comprehensive error types with detailed diagnostics
- **Network Configuration**: Automatic IP address, routing, and DNS configuration
- **Interface Binding**: Linux-specific interface binding for precise control
- **Broadcast Support**: Proper broadcast flag handling for ACK reception
- **Comprehensive Logging**: Structured logging with tracing for debugging
- **Memory Safe**: Written in Rust with zero unsafe code in core logic

## 🚀 Quick Start

### Installation

```bash
# From source
git clone https://github.com/your-org/herald.git
cd herald
cargo build --release

# Or install from crates.io (when published)
cargo install herald
```

### Basic Usage

```bash
# Run DHCP client on eth0 interface
sudo ./target/release/herald --interface eth0

# Or with short flag
sudo ./target/release/herald -i wlan0
```

**Note**: Root privileges are required for network interface binding and configuration.

### Library Usage

```rust
use herald::{DhcpClient, ClientConfig};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get MAC address for the interface
    let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
    
    // Create client configuration
    let config = ClientConfig::new("eth0".to_string(), mac_addr);
    
    // Initialize and run DHCP client
    let mut client = DhcpClient::new(config).await?;
    let lease = client.run().await?;
    
    println!("Obtained lease: {:?}", lease);
    Ok(())
}
```

## 📋 Requirements

- **Rust**: 1.70.0 or later
- **Operating System**: Linux (uses `SO_BINDTODEVICE`)
- **Privileges**: Root access for interface binding and configuration
- **Network**: Access to DHCP server on the target network

## 🏗️ Architecture

Herald is built with a modular architecture:

```
src/
├── lib.rs              # Library interface
├── main.rs             # CLI application entry point
├── client.rs           # Core DHCP client and state machine
├── config.rs           # Configuration structures
├── error.rs            # Error types and handling
├── network/
│   ├── mod.rs          # Socket creation and management
│   └── configurator.rs # Network interface configuration
└── v4/
    ├── mod.rs          # DHCPv4 module interface
    ├── handler.rs      # DHCPv4 state machine implementation
    ├── message.rs      # DHCP message construction
    └── tests.rs        # Unit tests
```

### State Machine

Herald implements a robust state machine following RFC 2131:

1. **Init** → **Selecting**: Broadcast DHCP DISCOVER
2. **Selecting** → **Requesting**: Receive DHCP OFFER, send DHCP REQUEST
3. **Requesting** → **Bound**: Receive DHCP ACK, configure interface
4. **Bound**: Lease active (future: renewal/rebinding)

## 🔧 Configuration

### Command Line Options

- `-i, --interface <INTERFACE>`: Network interface name (required)

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `RUST_LOG=debug`)

### Internal Configuration

Default values in `ClientConfig`:
- **Client Port**: 68 (DHCP client standard)
- **Server Port**: 67 (DHCP server standard)
- **Initial Timeout**: 5 seconds
- **Request Timeout**: 10 seconds
- **Broadcast Address**: 255.255.255.255

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests

# Run with coverage
cargo llvm-cov --html
```

## 📊 Performance

Herald is designed for efficiency:
- **Memory Usage**: Minimal allocations, zero-copy where possible
- **CPU Usage**: Asynchronous I/O prevents blocking
- **Network**: Optimized packet construction and parsing
- **Startup Time**: Fast initialization and DHCP negotiation

## 🔒 Security

- **Memory Safety**: Rust's ownership system prevents common vulnerabilities
- **Input Validation**: All network input is validated
- **Error Handling**: Graceful handling of malformed packets
- **Privilege Separation**: Minimal privilege requirements

## 🐛 Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   sudo ./target/release/herald -i eth0
   ```

2. **Interface Not Found**
   ```bash
   # List available interfaces
   ip link show
   ```

3. **No DHCP Response**
   ```bash
   # Check network connectivity
   # Verify DHCP server is running
   # Check firewall rules
   ```

### Debug Logging

```bash
RUST_LOG=debug sudo ./target/release/herald -i eth0
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/your-org/herald.git
cd herald
cargo build
cargo test
cargo clippy
cargo fmt
```

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- [dhcproto](https://crates.io/crates/dhcproto) for DHCP protocol implementation
- [tokio](https://tokio.rs/) for async runtime
- [socket2](https://crates.io/crates/socket2) for advanced socket options
- The Rust community for excellent tooling and libraries

## 📚 References

- [RFC 2131 - Dynamic Host Configuration Protocol](https://tools.ietf.org/html/rfc2131)
- [RFC 2132 - DHCP Options and BOOTP Vendor Extensions](https://tools.ietf.org/html/rfc2132)