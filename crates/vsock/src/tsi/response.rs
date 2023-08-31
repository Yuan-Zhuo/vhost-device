use crate::vhu_vsock::Result;

use strum::Display;
use virtio_vsock::packet::VsockPacket;
use vm_memory::bitmap::BitmapSlice;

use super::{proxy::ProxyID, VSOCK_HOST_CID};

#[derive(Debug, Clone, Display)]
pub enum TsiResponse {
    Connect(ConnectResult),
    Listen(ListenResult),
    Accept(AcceptResult),
    RecvStreamMsg(RecvStreamMsgInfo),
    RecvDgramMsg(RecvDgramMsgInfo),
    CreditUpdate(CreditUpdateResult),
    Op(OpResult),
    GetPeername(GetPeernameResult),
}

#[derive(Debug, Clone)]
pub struct ConnectResult {
    pub src_port: u32,
    pub dst_port: u32,
    pub result: i32,
}

#[derive(Debug, Clone)]
pub struct ListenResult {
    pub src_port: u32,
    pub dst_port: u32,
    pub result: i32,
}

#[derive(Debug, Clone)]
pub struct RecvStreamMsgInfo {
    pub src_port: u32,
    pub dst_port: u32,
    pub fwd_cnt: u32,
}

#[derive(Debug, Clone)]
pub struct RecvDgramMsgInfo {
    pub src_port: u32,
    pub dst_port: u32,
}

#[derive(Debug, Clone)]
pub struct CreditUpdateResult {
    pub src_port: u32,
    pub dst_port: u32,
    pub fwd_cnt: u32,
}

#[derive(Debug, Clone)]
pub struct AcceptResult {
    pub src_port: u32,
    pub dst_port: u32,
    pub result: i32,
}

#[derive(Debug, Clone)]
pub struct OpResult {
    pub src_port: u32,
    pub dst_port: u32,
}

#[derive(Debug, Clone)]
pub struct GetPeernameResult {
    pub src_port: u32,
    pub dst_port: u32,
    pub addr: u32,
    pub port: u16,
    pub result: i32,
}

pub fn init_proxy_pkt<'a, B: BitmapSlice>(
    id: &ProxyID,
    pkt: &mut VsockPacket<'a, B>,
    pkt_type: u16,
) -> Result<()> {
    pkt.set_src_cid(VSOCK_HOST_CID)
        .set_dst_cid(id.guest_cid)
        .set_type(pkt_type);

    Ok(())
}
