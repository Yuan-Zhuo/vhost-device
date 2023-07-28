// Transparent Socket Impersonation
pub mod proxy;
pub mod request;
pub mod response;
mod utils;

pub use request::TsiRequest;
pub use response::TsiResponse;
pub use utils::write_le_i32;

pub const VSOCK_HOST_CID: u64 = 2;

pub const SOCK_STREAM: u16 = 1;
pub const SOCK_DGRAM: u16 = 2;

pub const PROXY_PORT: u32 = 620;

pub const CONN_TX_BUF_SIZE: u32 = 8 << 20;

/// Reserved ports indicating the operation type
enum TsiReqOp {
    ProxyCreate,
    Connect,
    Getname,
    SendtoAddr,
    SendtoData,
    Listen,
    Accept,
    ProxyRelease,
    Unknown,
}

impl From<u32> for TsiReqOp {
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

#[allow(dead_code)]
pub enum TsiRespOp {
    Request,
    Response,
    Rst,
    ShutDown,
    Rw,
    CreditUpdate,
    CreditRequest,
}

impl Into<u16> for TsiRespOp {
    fn into(self) -> u16 {
        match self {
            TsiRespOp::Request => 1,
            TsiRespOp::Response => 2,
            TsiRespOp::Rst => 3,
            TsiRespOp::ShutDown => 4,
            TsiRespOp::Rw => 5,
            TsiRespOp::CreditUpdate => 6,
            TsiRespOp::CreditRequest => 7,
        }
    }
}
