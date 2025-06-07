#![allow(dead_code)]

use crate::{
    config::ClientConfig,
    error::HeraldError,
    v4::handler::DhcpV4Handler,
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::{
    net::UdpSocket,
    time::{self},
};

/// 状态机可以返回的动作，由客户端驱动器执行
#[derive(Debug)]
pub enum Action {
    Send(Vec<u8>, SocketAddr),
    StoreLease(Lease),
    Wait(Duration),
    Exit,
}

/// 状态机响应的外部事件
#[derive(Debug)]
pub enum Event<'a> {
    PacketReceived(&'a [u8]),
    Timeout,
}

/// 获得的租约信息
#[derive(Debug, Clone)]
pub struct Lease {
    pub offered_ip: Ipv4Addr,
    pub subnet_mask: Option<Ipv4Addr>,
    pub routers: Option<Vec<Ipv4Addr>>,
    pub dns_servers: Option<Vec<Ipv4Addr>>,
    pub lease_duration: Option<Duration>,
    pub server_identifier: Option<Ipv4Addr>,
}

/// DHCP 状态机的通用 Trait
pub trait DhcpStateMachine {
    /// 处理一个事件并返回下一个要执行的动作
    fn handle_event(&mut self, event: Event) -> Result<Action, HeraldError>;
    /// 获取当前状态的名称（用于日志记录）
    fn state_name(&self) -> &'static str;
}

pub struct DhcpClient {
    #[allow(dead_code)]
    config: ClientConfig,
    socket: UdpSocket,
    state_machine: Box<dyn DhcpStateMachine + Send>,
}

impl DhcpClient {
    pub async fn new(config: ClientConfig) -> Result<Self, HeraldError> {
        let socket =
            crate::network::new_tokio_socket_bound_to_device(&config.interface, config.client_port)?;

        let state_machine = Box::new(DhcpV4Handler::new(config.mac_address.clone()));

        Ok(Self {
            config,
            socket,
            state_machine,
        })
    }

    /// 等待响应或超时的通用方法
    async fn wait_for_response(&mut self, duration: Duration) -> Result<Action, HeraldError> {
        let mut buf = [0u8; 1500];
        match time::timeout(duration, self.socket.recv_from(&mut buf)).await {
            Ok(Ok((len, _addr))) => self
                .state_machine
                .handle_event(Event::PacketReceived(&buf[..len])),
            Ok(Err(e)) => Err(HeraldError::Io(e)),
            Err(_) => {
                // 超时
                self.state_machine.handle_event(Event::Timeout)
            }
        }
    }

    pub async fn run(&mut self) -> Result<Lease, HeraldError> {
        // 启动状态机
        let mut next_action = self.state_machine.handle_event(Event::Timeout)?;

        loop {
            tracing::info!(
                "State: {}, Action: {:?}",
                self.state_machine.state_name(),
                next_action
            );

            match next_action {
                Action::Send(packet, addr) => {
                    self.socket.send_to(&packet, addr).await?;
                    // 发送后，等待响应或超时，使用默认的超时时间
                    let timeout_duration = Duration::from_secs(5); // 5秒超时
                    next_action = self.wait_for_response(timeout_duration).await?;
                }
                Action::Wait(duration) => {
                    next_action = self.wait_for_response(duration).await?;
                }
                Action::StoreLease(lease) => {
                    tracing::info!("DHCP Bind Successful! Lease: {:?}", lease);
                    // 在这里，我们可以调用 NetworkConfigurator 来应用设置
                    return Ok(lease);
                }
                Action::Exit => {
                    return Err(HeraldError::Critical(
                        "State machine exited prematurely".to_string(),
                    ));
                }
            }
        }
    }
}