pub mod tcp;
pub mod udp;

use std::{collections::VecDeque, os::fd::AsRawFd};

use nix::errno::Errno;

use super::{
    request::{ConnectConfig, ListenConfig},
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

pub trait Proxy: Send + AsRawFd {
    fn id(&self) -> &ProxyID;
    fn resp_queue(&mut self) -> &mut VecDeque<TsiResponse>;

    // Tsi Requsets
    fn connect(&mut self, connect_config: ConnectConfig) -> Result<(), Errno>;
    fn listen(&mut self, listen_config: ListenConfig) -> Result<(), Errno>;

    // Proxy Events
    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Errno>;
}
