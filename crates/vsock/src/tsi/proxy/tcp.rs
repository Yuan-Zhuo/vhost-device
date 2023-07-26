use log::info;
use nix::{
    errno::Errno,
    sys::socket::{
        bind, connect, listen, recv, socket, AddressFamily, MsgFlags, SockFlag, SockType,
        SockaddrStorage,
    },
};
use std::{
    collections::VecDeque,
    net::{IpAddr, SocketAddr},
    os::unix::io::{AsRawFd, RawFd},
};

use super::{Proxy, ProxyID, ProxyStatus};
use crate::tsi::{
    request::{ConnectConfig, ListenConfig},
    response::TsiResponse,
};

pub struct TcpProxy {
    pub id: ProxyID,
    pub fd: RawFd,
    pub status: ProxyStatus,
    pub resp_queue: VecDeque<TsiResponse>,
}

#[allow(dead_code)]
impl TcpProxy {
    pub fn new(id: ProxyID) -> Result<Self, Errno> {
        let fd = socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::SOCK_NONBLOCK,
            None,
        )?;

        Ok(TcpProxy {
            id,
            fd,
            status: ProxyStatus::Idle,
            resp_queue: VecDeque::new(),
        })
    }
}

impl Proxy for TcpProxy {
    fn id(&self) -> &ProxyID {
        &self.id
    }

    fn resp_queue(&mut self) -> &mut VecDeque<TsiResponse> {
        &mut self.resp_queue
    }

    fn connect(&mut self, connect_config: ConnectConfig) -> Result<(), Errno> {
        info!("SNOOPY connect: {:?}", connect_config);

        connect(
            self.fd,
            &SockaddrStorage::from(SocketAddr::new(
                IpAddr::V4(connect_config.addr),
                connect_config.port,
            )),
        )?;

        self.status = ProxyStatus::Connected;

        Ok(())
    }

    fn listen(&mut self, listen_config: ListenConfig) -> Result<(), Errno> {
        bind(
            self.fd,
            &SockaddrStorage::from(SocketAddr::new(
                IpAddr::V4(listen_config.addr),
                listen_config.port,
            )),
        )?;

        listen(self.fd, listen_config.backlog as usize)?;

        Ok(())
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Errno> {
        let len = recv(self.fd, buffer, MsgFlags::empty())?;

        if len == buffer.len() {
            self.resp_queue.push_back(TsiResponse::RecvData);
        }

        Ok(len)
    }
}

impl AsRawFd for TcpProxy {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
