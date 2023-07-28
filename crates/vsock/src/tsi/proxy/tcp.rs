use nix::{
    errno::Errno,
    sys::socket::{
        bind, connect, listen, recv, send, socket, AddressFamily, MsgFlags, SockFlag, SockType,
        SockaddrStorage,
    },
};
use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::Wrapping,
    os::unix::io::{AsRawFd, RawFd},
};

use super::{Proxy, ProxyID, ProxyStatus, ProxyType};
use crate::tsi::{
    request::{ConnectConfig, ListenConfig, SendMsgConfig},
    response::{CreditUpdateResult, RecvStreamMsgInfo, TsiResponse},
    CONN_TX_BUF_SIZE,
};

const LOCALHOST_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

pub struct TcpProxy {
    pub id: ProxyID,
    pub fd: RawFd,
    pub status: ProxyStatus,
    pub resp_queue: VecDeque<TsiResponse>,
    pub tx_cnt: Wrapping<u32>,
    pub last_tx_cnt_sent: Wrapping<u32>,
    pub rx_cnt: Wrapping<u32>,
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
            tx_cnt: Wrapping(0),
            last_tx_cnt_sent: Wrapping(0),
            rx_cnt: Wrapping(0),
        })
    }
}

impl Proxy for TcpProxy {
    fn type_(&self) -> ProxyType {
        ProxyType::Stream
    }

    fn id(&self) -> &ProxyID {
        &self.id
    }

    fn fwd_cnt(&self) -> u32 {
        self.tx_cnt.0
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

    fn recv(&mut self, buffer: &mut [u8]) -> Result<u32, Errno> {
        let len = recv(self.fd, buffer, MsgFlags::empty())?;
        self.rx_cnt += len as u32;

        if len == buffer.len() {
            self.resp_queue
                .push_back(TsiResponse::RecvStreamMsg(RecvStreamMsgInfo {
                    src_port: self.id.local_port,
                    dst_port: self.id.peer_port,
                    fwd_cnt: self.tx_cnt.0,
                }));
        }

        Ok(len as u32)
    }

    fn send(&mut self, send_msg_config: SendMsgConfig) -> Result<bool, Errno> {
        let len = send(self.fd, &send_msg_config.data, MsgFlags::MSG_NOSIGNAL)? as u32;
        self.tx_cnt += len;

        let credit_update =
            len > 0 && (self.tx_cnt - self.last_tx_cnt_sent).0 >= (CONN_TX_BUF_SIZE / 2);

        if credit_update {
            self.last_tx_cnt_sent = self.tx_cnt;
            self.resp_queue
                .push_back(TsiResponse::CreditUpdate(CreditUpdateResult {
                    src_port: self.id.local_port,
                    dst_port: self.id.peer_port,
                    fwd_cnt: self.tx_cnt.0,
                }));
        }

        Ok(credit_update)
    }
}

impl AsRawFd for TcpProxy {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
