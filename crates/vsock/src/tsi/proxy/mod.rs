pub mod tcp;
pub mod udp;

use std::{collections::VecDeque, os::fd::AsRawFd};

use nix::{errno::Errno, sys::socket::SockaddrIn};

use super::{
    request::{AcceptConfig, ConnectConfig, ListenConfig, OpResponseConfig, SendMsgConfig},
    TsiResponse,
};

/// Identify a proxy by guest cid and port numbers
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ProxyID {
    pub guest_cid: u64,
    pub peer_port: u32,
    pub local_port: u32,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum ProxyStatus {
    Idle,
    Connected,
    Closed,
    Listen,
    ReverseInit,
}

impl ProxyID {
    pub fn new(guest_cid: u64, peer_port: u32, local_port: u32) -> ProxyID {
        ProxyID {
            guest_cid,
            peer_port,
            local_port,
        }
    }
}

pub enum ProxyType {
    Stream,
    Dgram,
}

pub type Result<T> = core::result::Result<T, Errno>;

pub trait Proxy: Send + AsRawFd {
    fn type_(&self) -> ProxyType;
    fn status(&self) -> ProxyStatus;
    fn id(&self) -> &ProxyID;
    fn control_port(&self) -> u32;
    fn fwd_cnt(&self) -> u32;
    fn resp_queue(&mut self) -> &mut VecDeque<TsiResponse>;

    // Tsi Requsets
    fn connect(&mut self, connect_config: ConnectConfig) -> Result<()>;
    fn listen(&mut self, listen_config: ListenConfig) -> Result<()>;
    fn send(&mut self, send_msg_config: SendMsgConfig) -> Result<bool>;
    fn check_accept(&mut self, accept_config: AcceptConfig) -> Option<i32>;
    fn ack_accept(&mut self, op_response_config: OpResponseConfig) -> Result<()>;
    fn getpeername(&self) -> Result<SockaddrIn>;

    // Proxy Events
    fn recv(&mut self, buffer: &mut [u8]) -> Result<u32>;
    fn accept(&mut self, accept_id: ProxyID) -> Result<Box<dyn Proxy>>;
}
