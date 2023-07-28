use nix::{
    errno::Errno,
    fcntl::{fcntl, FcntlArg, OFlag},
    sys::socket::{
        connect, recv, send, socket, AddressFamily, MsgFlags, SockFlag, SockType, SockaddrStorage,
    },
};
use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::unix::io::{AsRawFd, RawFd},
};

use super::{Proxy, ProxyID, ProxyStatus, ProxyType};
use crate::tsi::{
    request::{ConnectConfig, ListenConfig, SendMsgConfig},
    response::{RecvDgramMsgInfo, TsiResponse},
};

const LOCALHOST_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

pub struct UdpProxy {
    pub id: ProxyID,
    pub fd: RawFd,
    pub status: ProxyStatus,
    pub resp_queue: VecDeque<TsiResponse>,
}

impl UdpProxy {
    pub fn new(id: ProxyID) -> Result<Self, Errno> {
        let fd = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        )?;

        let flags = OFlag::from_bits(fcntl(fd, FcntlArg::F_GETFL)?).unwrap();
        fcntl(fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))?;

        Ok(UdpProxy {
            id,
            fd,
            status: ProxyStatus::Idle,
            resp_queue: VecDeque::new(),
        })
    }
}

impl Proxy for UdpProxy {
    fn type_(&self) -> ProxyType {
        ProxyType::Dgram
    }

    fn id(&self) -> &ProxyID {
        &self.id
    }

    fn fwd_cnt(&self) -> u32 {
        unreachable!()
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
        connect(self.fd, &addr)?;

        self.status = ProxyStatus::Connected;

        Ok(())
    }

    fn listen(&mut self, _listen_config: ListenConfig) -> Result<(), Errno> {
        unreachable!()
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<u32, Errno> {
        let len = recv(self.fd, buffer, MsgFlags::empty())?;

        if len == buffer.len() {
            self.resp_queue
                .push_back(TsiResponse::RecvDgramMsg(RecvDgramMsgInfo {
                    src_port: 0,
                    dst_port: self.id.peer_port,
                }));
        }

        Ok(len as u32)
    }

    fn send(&mut self, send_msg_config: SendMsgConfig) -> Result<bool, Errno> {
        send(self.fd, &send_msg_config.data, MsgFlags::MSG_NOSIGNAL)?;

        Ok(false)
    }
}

impl AsRawFd for UdpProxy {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
