#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use dhcproto::{v4, Decodable, Decoder};
    use std::net::Ipv4Addr;

    #[test]
    fn test_build_dhcp_discover() {
        let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
        let xid = 0x12345678;
        
        let packet = build_dhcp_discover(&mac_addr, xid).unwrap();
        
        // Decode the packet to verify it's valid
        let mut decoder = Decoder::new(&packet);
        let msg = v4::Message::decode(&mut decoder).unwrap();
        
        assert_eq!(msg.xid(), xid);
        assert_eq!(msg.chaddr(), &mac_addr[..]);
        assert_eq!(msg.opcode(), v4::Opcode::BootRequest);
        
        // Check for DHCP message type
        let msg_type = msg.opts().get(v4::OptionCode::MessageType);
        assert!(matches!(msg_type, Some(v4::DhcpOption::MessageType(v4::MessageType::Discover))));
        
        // Check for client identifier
        let client_id = msg.opts().get(v4::OptionCode::ClientIdentifier);
        assert!(client_id.is_some());
    }

    #[test]
    fn test_build_dhcp_request() {
        let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
        let xid = 0x87654321;
        let offered_ip = Ipv4Addr::new(192, 168, 1, 100);
        let server_ip = Ipv4Addr::new(192, 168, 1, 1);
        
        let packet = build_dhcp_request(&mac_addr, xid, offered_ip, server_ip).unwrap();
        
        // Decode the packet to verify it's valid
        let mut decoder = Decoder::new(&packet);
        let msg = v4::Message::decode(&mut decoder).unwrap();
        
        assert_eq!(msg.xid(), xid);
        assert_eq!(msg.chaddr(), &mac_addr[..]);
        assert_eq!(msg.opcode(), v4::Opcode::BootRequest);
        
        // Check for DHCP message type
        let msg_type = msg.opts().get(v4::OptionCode::MessageType);
        assert!(matches!(msg_type, Some(v4::DhcpOption::MessageType(v4::MessageType::Request))));
        
        // Check for requested IP address
        let requested_ip = msg.opts().get(v4::OptionCode::RequestedIpAddress);
        assert!(matches!(requested_ip, Some(v4::DhcpOption::RequestedIpAddress(ip)) if *ip == offered_ip));
        
        // Check for server identifier
        let server_id = msg.opts().get(v4::OptionCode::ServerIdentifier);
        assert!(matches!(server_id, Some(v4::DhcpOption::ServerIdentifier(ip)) if *ip == server_ip));
        
        // Check broadcast flag is set
        assert!(msg.flags().broadcast());
    }

    #[test]
    fn test_dhcp_v4_handler_creation() {
        let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
        let handler = DhcpV4Handler::new(mac_addr.clone());
        
        assert_eq!(handler.state_name(), "Init");
    }

    #[test]
    fn test_dhcp_v4_handler_init_transition() {
        let mac_addr = Bytes::from_static(&[0x00, 0x0c, 0x29, 0xa8, 0x92, 0xf4]);
        let mut handler = DhcpV4Handler::new(mac_addr);
        
        let action = handler.handle_event(crate::client::Event::Timeout).unwrap();
        
        match action {
            crate::client::Action::Send(packet, addr) => {
                assert!(!packet.is_empty());
                assert_eq!(addr.port(), 67);
            }
            _ => panic!("Expected Send action"),
        }
        
        assert_eq!(handler.state_name(), "Selecting");
    }
}