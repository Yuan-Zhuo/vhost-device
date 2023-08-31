// Transparent Socket Impersonation
pub mod proxy;
pub mod request;
pub mod response;
mod utils;

pub use request::TsiRequest;
pub use response::TsiResponse;
use strum::Display;
pub use utils::{write_be_u32, write_le_i32, write_le_u16};

pub const VSOCK_HOST_CID: u64 = 2;

pub const SOCK_STREAM: u16 = 1;
pub const SOCK_DGRAM: u16 = 2;
pub const SOCK_TSI_STREAM: u16 = 7;
pub const SOCK_TSI_DGRAM: u16 = 8;

pub const PROXY_PORT: u32 = 620;

pub const CONN_TX_BUF_SIZE: u32 = 8 << 20;

/// Reserved ports indicating the operation type
#[derive(PartialEq, Eq, Display, Debug)]
pub enum TsiReqCtlOp {
    ProxyCreate = 1024,
    Connect,
    Getname,
    SendtoAddr,
    SendtoData,
    Listen,
    Accept,
    ProxyRelease,
    Unknown,
}

impl From<u32> for TsiReqCtlOp {
    fn from(port: u32) -> Self {
        match port {
            1024 => Self::ProxyCreate,
            1025 => Self::Connect,
            1026 => Self::Getname,
            1027 => Self::SendtoAddr,
            1028 => Self::SendtoData,
            1029 => Self::Listen,
            1030 => Self::Accept,
            1031 => Self::ProxyRelease,
            _ => Self::Unknown,
        }
    }
}

/// Op mappings
pub enum TsiStreamOp {
    Request = 1,
    Response,
    Rst,
    ShutDown,
    Rw,
    CreditUpdate,
    CreditRequest,
    Unknown,
}

impl From<u16> for TsiStreamOp {
    fn from(op: u16) -> Self {
        match op {
            1 => Self::Request,
            2 => Self::Response,
            3 => Self::Rst,
            4 => Self::ShutDown,
            5 => Self::Rw,
            6 => Self::CreditUpdate,
            7 => Self::CreditRequest,
            _ => Self::Unknown,
        }
    }
}

#[allow(dead_code)]
pub enum TsiRespCtlOp {
    Request = 1,
    Response,
    Rst,
    ShutDown,
    Rw,
    CreditUpdate,
    CreditRequest,
}

impl Into<u16> for TsiRespCtlOp {
    fn into(self) -> u16 {
        match self {
            TsiRespCtlOp::Request => 1,
            TsiRespCtlOp::Response => 2,
            TsiRespCtlOp::Rst => 3,
            TsiRespCtlOp::ShutDown => 4,
            TsiRespCtlOp::Rw => 5,
            TsiRespCtlOp::CreditUpdate => 6,
            TsiRespCtlOp::CreditRequest => 7,
        }
    }
}
