#![allow(dead_code)]

use super::message::build_dhcp_discover;
use crate::{
    client::{Action, DhcpStateMachine, Event},
    error::HeraldError,
};
use bytes::Bytes;
use dhcproto::{v4, Decodable};
use std::{net::SocketAddr, time::Duration};

#[derive(Debug, PartialEq, Clone, Copy)]
enum DhcpV4State {
    Init,
    Selecting,
    Requesting,
    Bound,
}

pub struct DhcpV4Handler {
    state: DhcpV4State,
    mac_address: Bytes,
    xid: u32,
    offer: Option<v4::Message>,
}

impl DhcpV4Handler {
    pub fn new(mac_address: Bytes) -> Self {
        Self {
            state: DhcpV4State::Init,
            mac_address,
            xid: rand::random(),
            offer: None,
        }
    }

    // 私有辅助函数来处理特定的状态转换
    fn handle_init(&mut self) -> Result<Action, HeraldError> {
        self.state = DhcpV4State::Selecting;
        let discover_packet = build_dhcp_discover(&self.mac_address, self.xid)?;
        let broadcast_addr = "255.255.255.255:67".parse::<SocketAddr>().unwrap();
        Ok(Action::Send(discover_packet, broadcast_addr))
    }

    fn handle_selecting(&mut self, event: Event) -> Result<Action, HeraldError> {
        match event {
            Event::PacketReceived(data) => {
                let msg = v4::Message::decode(&mut v4::Decoder::new(data))
                    .map_err(|e| HeraldError::Protocol(Box::new(e)))?;

                if msg.xid() == self.xid {
                    if let Some(v4::DhcpOption::MessageType(v4::MessageType::Offer)) =
                        msg.opts().get(v4::OptionCode::MessageType)
                    {
                        self.offer = Some(msg);
                        self.state = DhcpV4State::Requesting;
                        return self.handle_requesting();
                    }
                }
                // 不是我们想要的包，继续等待
                Ok(Action::Wait(Duration::from_secs(5)))
            }
            Event::Timeout => {
                // 超时，重新发送 Discover
                self.state = DhcpV4State::Init;
                self.handle_init()
            }
        }
    }

    fn handle_requesting(&mut self) -> Result<Action, HeraldError> {
        // ... 此处实现请求逻辑 ...
        // ... 成功后，将状态转换为 Bound 并返回 Action::Wait(timeout_for_ack) ...
        unimplemented!()
    }
}

impl DhcpStateMachine for DhcpV4Handler {
    fn state_name(&self) -> &'static str {
        match self.state {
            DhcpV4State::Init => "Init",
            DhcpV4State::Selecting => "Selecting",
            DhcpV4State::Requesting => "Requesting",
            DhcpV4State::Bound => "Bound",
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<Action, HeraldError> {
        tracing::debug!("Handling event {:?} in state {:?}", event, self.state);
        match self.state {
            DhcpV4State::Init => self.handle_init(),
            DhcpV4State::Selecting => self.handle_selecting(event),
            // ... 其他状态 ...
            _ => unimplemented!(),
        }
    }
}