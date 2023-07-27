use nix::{
    errno::Errno,
    sys::socket::{
        bind, connect, listen, recv, socket, AddressFamily, MsgFlags, SockFlag, SockType,
        SockaddrStorage,
    },
};
use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::unix::io::{AsRawFd, RawFd},
};

use super::{Proxy, ProxyID, ProxyStatus};
use crate::tsi::{
    request::{ConnectConfig, ListenConfig, SendMsgConfig},
    response::{RecvMsgInfo, TsiResponse},
};

const LOCALHOST_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

pub struct TcpProxy {
    pub id: ProxyID,
    pub fd: RawFd,
    pub status: ProxyStatus,
    pub resp_queue: VecDeque<TsiResponse>,
}

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
        // SNOOPY HACK HERE:
        //     Replace ip with localhost for debugging.
        let addr = SockaddrStorage::from(SocketAddr::new(
            IpAddr::V4(LOCALHOST_ADDR),
            connect_config.port,
        ));

        match connect(self.fd, &addr) {
            Ok(_) | Err(Errno::EINPROGRESS) => {
                self.status = ProxyStatus::Connected;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn listen(&mut self, listen_config: ListenConfig) -> Result<(), Errno> {
        let addr = SockaddrStorage::from(SocketAddr::new(
            IpAddr::V4(listen_config.addr),
            listen_config.port,
        ));

        bind(self.fd, &addr)?;

        listen(self.fd, listen_config.backlog as usize)?;

        Ok(())
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Errno> {
        let len = recv(self.fd, buffer, MsgFlags::empty())?;

        if len == buffer.len() {
            self.resp_queue.push_back(TsiResponse::RecvMsg(RecvMsgInfo {
                src_port: 0,
                dst_port: self.id.peer_port,
            }));
        }

        Ok(len)
    }

    fn send(&mut self, _send_msg_config: SendMsgConfig) -> Result<usize, Errno> {
        todo!()
    }
}

impl AsRawFd for TcpProxy {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
