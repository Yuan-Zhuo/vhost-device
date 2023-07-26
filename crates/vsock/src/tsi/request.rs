use crate::tsi::utils::{read_be_u16, read_le_i32, read_le_u16, read_le_u32, read_le_u8};
use crate::tsi::TsiReqOp;
use crate::vhu_vsock::{Error, Result};

use log::info;
use std::convert::TryFrom;
use std::net::Ipv4Addr;
use strum::Display;
use virtio_vsock::packet::VsockPacket;
use vm_memory::bitmap::BitmapSlice;

#[derive(Debug, Clone, Display)]
pub enum TsiRequest {
    PorxyCreate(ProxyCreateConfig),
    Connect(ConnectConfig),
    Getname(GetnameConfig),
    SendtoAddr(SendtoAddrConfig),
    SendtoData,
    Listen(ListenConfig),
    Accept(AcceptConfig),
    ProxyRelease(ProxyReleaseConfig),
}

#[derive(Debug, Clone)]
pub struct ProxyCreateConfig {
    pub peer_port: u32,
    pub type_: u16,
}

#[derive(Debug, Clone)]
pub struct ConnectConfig {
    pub peer_port: u32,
    pub addr: Ipv4Addr,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct GetnameConfig {
    pub peer_port: u32,
    pub local_port: u32,
    pub peer: u32,
}

#[derive(Debug, Clone)]
pub struct SendtoAddrConfig {
    pub peer_port: u32,
    pub addr: Ipv4Addr,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct ListenConfig {
    pub peer_port: u32,
    pub addr: Ipv4Addr,
    pub port: u16,
    pub vm_port: u32,
    pub backlog: i32,
}

#[derive(Debug, Clone)]
pub struct AcceptConfig {
    pub peer_port: u32,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct ProxyReleaseConfig {
    pub peer_port: u32,
    pub local_port: u32,
}

/// Convert VsockPacket to different types of TsiRequest
impl<'a, B: BitmapSlice> TryFrom<&VsockPacket<'a, B>> for TsiRequest {
    type Error = Error;

    fn try_from(pkt: &VsockPacket<'a, B>) -> Result<Self> {
        let data_slice = pkt.data_slice().ok_or(Error::PktBufMissing)?;

        let len = data_slice.len();
        let mut buffer = vec![0u8; len];
        data_slice.copy_to(&mut buffer);
        info!("SNOOPY data_slice: {:?}", buffer);

        match pkt.dst_port().into() {
            TsiReqOp::ProxyCreate => Ok(Self::PorxyCreate(ProxyCreateConfig {
                peer_port: read_le_u32(data_slice, 0)?,
                type_: read_le_u16(data_slice, 4)?,
            })),
            TsiReqOp::Connect => Ok(Self::Connect(ConnectConfig {
                peer_port: read_le_u32(data_slice, 0)?,
                addr: Ipv4Addr::new(
                    read_le_u8(data_slice, 4)?,
                    read_le_u8(data_slice, 5)?,
                    read_le_u8(data_slice, 6)?,
                    read_le_u8(data_slice, 7)?,
                ),
                port: read_be_u16(data_slice, 8)?,
            })),
            TsiReqOp::Getname => Ok(Self::Getname(GetnameConfig {
                peer_port: read_le_u32(data_slice, 0)?,
                local_port: read_le_u32(data_slice, 4)?,
                peer: read_le_u32(data_slice, 8)?,
            })),
            TsiReqOp::SendtoAddr => Ok(Self::SendtoAddr(SendtoAddrConfig {
                peer_port: read_le_u32(data_slice, 0)?,
                addr: Ipv4Addr::new(
                    read_le_u8(data_slice, 4)?,
                    read_le_u8(data_slice, 5)?,
                    read_le_u8(data_slice, 6)?,
                    read_le_u8(data_slice, 7)?,
                ),
                port: read_le_u16(data_slice, 8)?,
            })),
            TsiReqOp::SendtoData => Ok(Self::SendtoData),
            TsiReqOp::Listen => Ok(Self::Listen(ListenConfig {
                peer_port: read_le_u32(data_slice, 0)?,
                addr: Ipv4Addr::new(
                    read_le_u8(data_slice, 4)?,
                    read_le_u8(data_slice, 5)?,
                    read_le_u8(data_slice, 6)?,
                    read_le_u8(data_slice, 7)?,
                ),
                port: read_le_u16(data_slice, 8)?,
                vm_port: read_le_u32(data_slice, 10)?,
                backlog: read_le_i32(data_slice, 14)?,
            })),
            TsiReqOp::Accept => Ok(Self::Accept(AcceptConfig {
                peer_port: read_le_u32(data_slice, 0)?,
                flags: read_le_u32(data_slice, 4)?,
            })),
            TsiReqOp::ProxyRelease => Ok(Self::ProxyRelease(ProxyReleaseConfig {
                peer_port: read_le_u32(data_slice, 0)?,
                local_port: read_le_u32(data_slice, 4)?,
            })),
            TsiReqOp::Unknown => {
                unreachable!()
            }
        }
    }
}
