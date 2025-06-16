use bytes::Bytes;
use herald::{ClientConfig, DhcpClient};
use std::time::Duration;

#[tokio::test]
async fn test_client_creation() {
    let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
    let config = ClientConfig::new("lo".to_string(), mac_addr);

    // This should not panic, even if we can't actually bind to the interface
    // in a test environment
    let result = DhcpClient::new(config).await;

    // We expect this to potentially fail in CI, but it shouldn't panic
    match result {
        Ok(_) => {
            // Success case - we managed to create the client
        }
        Err(e) => {
            // Expected in CI environments without proper network setup
            println!("Expected error in test environment: {}", e);
        }
    }
}

#[test]
fn test_config_creation() {
    let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
    let config = ClientConfig::new("eth0".to_string(), mac_addr.clone());

    assert_eq!(config.interface, "eth0");
    assert_eq!(config.mac_address, mac_addr);
    assert_eq!(config.client_port, 68);
    assert_eq!(config.server_port, 67);
    assert_eq!(config.initial_timeout, Duration::from_secs(5));
    assert_eq!(config.request_timeout, Duration::from_secs(10));
}

#[test]
fn test_mac_address_handling() {
    let mac_bytes = vec![0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4];
    let mac_addr = Bytes::from(mac_bytes.clone());

    assert_eq!(mac_addr.len(), 6);
    assert_eq!(mac_addr.to_vec(), mac_bytes);
}
