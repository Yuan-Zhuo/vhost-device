pub mod tcp;
pub mod udp;

use std::{collections::VecDeque, os::fd::AsRawFd};

use nix::errno::Errno;

use super::{
    request::{ConnectConfig, ListenConfig, SendMsgConfig},
    TsiResponse,
};

/// Identify a proxy by port number
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

pub trait Proxy: Send + AsRawFd {
    fn type_(&self) -> ProxyType;
    fn id(&self) -> &ProxyID;
    fn fwd_cnt(&self) -> u32;
    fn resp_queue(&mut self) -> &mut VecDeque<TsiResponse>;

    // Tsi Requsets
    fn connect(&mut self, connect_config: ConnectConfig) -> Result<(), Errno>;
    fn listen(&mut self, listen_config: ListenConfig) -> Result<(), Errno>;
    fn send(&mut self, send_msg_config: SendMsgConfig) -> Result<bool, Errno>;

    // Proxy Events
    fn recv(&mut self, buffer: &mut [u8]) -> Result<u32, Errno>;
}
