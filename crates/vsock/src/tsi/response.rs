use crate::vhu_vsock::{Result, VSOCK_TYPE_DGRAM};

use strum::Display;
use virtio_vsock::packet::VsockPacket;
use vm_memory::bitmap::BitmapSlice;

use super::{proxy::ProxyID, VSOCK_HOST_CID};

#[derive(Debug, Clone, Display)]
pub enum TsiResponse {
    Connect(ConnectResult),
    Listen(ListenResult),
    RecvData,
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

pub fn init_proxy_pkt<'a, B: BitmapSlice>(
    id: &ProxyID,
    pkt: &mut VsockPacket<'a, B>,
) -> Result<()> {
    pkt.set_src_cid(VSOCK_HOST_CID)
        .set_dst_cid(id.guest_cid)
        .set_type(VSOCK_TYPE_DGRAM);

    Ok(())
}
