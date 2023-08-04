use nix::{
    errno::Errno,
    sys::socket::{
        accept, bind, connect, getpeername, listen, recv, send, socket, AddressFamily, MsgFlags,
        SockFlag, SockType, SockaddrIn, SockaddrStorage,
    },
};
use std::{
    collections::VecDeque,
    net::{IpAddr, SocketAddr},
    num::Wrapping,
    os::unix::io::{AsRawFd, RawFd},
};

use super::{Proxy, ProxyID, ProxyStatus, ProxyType, Result};
use crate::tsi::{
    request::{AcceptConfig, ConnectConfig, ListenConfig, OpResponseConfig, SendMsgConfig},
    response::{CreditUpdateResult, RecvStreamMsgInfo, TsiResponse},
    CONN_TX_BUF_SIZE,
};

pub struct TcpProxy {
    id: ProxyID,
    fd: RawFd,
    status: ProxyStatus,
    resp_queue: VecDeque<TsiResponse>,
    control_port: u32,
    tx_cnt: Wrapping<u32>,
    last_tx_cnt_sent: Wrapping<u32>,
    rx_cnt: Wrapping<u32>,
    pending_accepts: u32,
}

impl TcpProxy {
    pub fn new(id: ProxyID, control_port: u32) -> Result<Self> {
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
            control_port,
            resp_queue: VecDeque::new(),
            tx_cnt: Wrapping(0),
            last_tx_cnt_sent: Wrapping(0),
            rx_cnt: Wrapping(0),
            pending_accepts: 0,
        })
    }
}

impl Proxy for TcpProxy {
    fn type_(&self) -> ProxyType {
        ProxyType::Stream
    }

    fn status(&self) -> ProxyStatus {
        self.status
    }

    fn id(&self) -> &ProxyID {
        &self.id
    }

    fn control_port(&self) -> u32 {
        self.control_port
    }

    fn fwd_cnt(&self) -> u32 {
        self.tx_cnt.0
    }

    fn resp_queue(&mut self) -> &mut VecDeque<TsiResponse> {
        &mut self.resp_queue
    }

    fn connect(&mut self, connect_config: ConnectConfig) -> Result<()> {
        let addr = SockaddrStorage::from(SocketAddr::new(
            IpAddr::V4(connect_config.addr),
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

    fn listen(&mut self, listen_config: ListenConfig) -> Result<()> {
        let addr = SockaddrStorage::from(SocketAddr::new(
            IpAddr::V4(listen_config.addr),
            listen_config.port,
        ));

        bind(self.fd, &addr)?;

        listen(self.fd, listen_config.backlog as usize)?;

        self.status = ProxyStatus::Listen;

        Ok(())
    }

    fn check_accept(&mut self, accept_config: AcceptConfig) -> Option<i32> {
        let result = if self.pending_accepts > 0 {
            self.pending_accepts -= 1;
            0
        } else if (accept_config.flags & SockFlag::SOCK_NONBLOCK.bits() as u32) != 0 {
            -(Errno::EWOULDBLOCK as i32)
        } else {
            return None;
        };

        Some(result)
    }

    fn ack_accept(&mut self, op_response_config: OpResponseConfig) -> Result<()> {
        self.tx_cnt = Wrapping(op_response_config.fwd_cnt);
        self.status = ProxyStatus::Connected;

        Ok(())
    }

    fn accept(&mut self, accept_id: ProxyID) -> Result<Box<dyn Proxy>> {
        let accept_fd = accept(self.fd)?;

        Ok(Box::new(TcpProxy {
            id: accept_id,
            control_port: self.control_port,
            fd: accept_fd,
            status: ProxyStatus::ReverseInit,
            resp_queue: VecDeque::new(),
            tx_cnt: Wrapping(0),
            last_tx_cnt_sent: Wrapping(0),
            rx_cnt: Wrapping(0),
            pending_accepts: 0,
        }))
    }

    fn getpeername(&self) -> Result<SockaddrIn> {
        let peername = getpeername::<SockaddrStorage>(self.fd)?;
        let sock_addr = peername
            .as_sockaddr_in()
            .ok_or(Errno::EADDRNOTAVAIL)?
            .clone();

        Ok(sock_addr)
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<u32> {
        let len = recv(self.fd, buffer, MsgFlags::empty())?;
        self.rx_cnt += len as u32;

        if len == 0 {
            self.status = ProxyStatus::Closed;
            return Err(Errno::ENODATA);
        }

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

    fn send(&mut self, send_msg_config: SendMsgConfig) -> Result<bool> {
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
