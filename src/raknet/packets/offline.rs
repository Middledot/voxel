use std::net::SocketAddr;

use crate::raknet::objects::MsgBuffer;

use super::obj::{FromBuffer, ToBuffer};

pub struct OfflinePing {
    pub timestamp: i64,
    pub magic: [u8; 16],
    pub client_guid: i64,
}

impl FromBuffer for OfflinePing {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        // keep client_timestamp instead of timestamp the
        // distinction might be important in the future idk
        let client_timestamp = buf.read_i64_be_bytes();
        let magic = buf.read_magic();
        let client_guid = buf.read_i64_be_bytes();

        Self {
            timestamp: client_timestamp,
            magic,
            client_guid,
        }
    }
}

pub struct OfflinePong {
    pub timestamp: i64,
    pub server_guid: i64,
    pub magic: [u8; 16],
    pub server_name: String,
}

impl ToBuffer for OfflinePong {
    fn to_buffer(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_i64_be_bytes(&self.timestamp);
        buf.write_i64_be_bytes(&self.server_guid);
        buf.write_magic(&self.magic);
        buf.write_string(&self.server_name);

        buf
    }
}

pub struct OfflineConnReq1 {
    pub magic: [u8; 16],
    pub protocol: u8, // mysterious magical mystical value, unknown use (always 0x11)
    pub mtu: i16,
}

impl FromBuffer for OfflineConnReq1 {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        let magic = buf.read_magic();
        let protocol = buf.read_byte();
        let mtu = (buf.len_rest() + 46) as i16;

        Self {
            magic,
            protocol,
            mtu,
        }
    }
}

pub struct OfflineConnRep1 {
    pub magic: [u8; 16],
    pub server_guid: i64,
    pub use_security: bool,
    pub mtu: i16,
}

impl ToBuffer for OfflineConnRep1 {
    fn to_buffer(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_magic(&self.magic);
        buf.write_i64_be_bytes(&self.server_guid);
        buf.write_byte(self.use_security as u8);
        buf.write_i16_be_bytes(&self.mtu);

        buf
    }
}

pub struct OfflineConnReq2 {
    pub magic: [u8; 16],
    pub server_address: SocketAddr,
    pub mtu: i16,
    pub client_guid: i64,
}

impl FromBuffer for OfflineConnReq2 {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        let magic = buf.read_magic();
        let server_address = buf.read_address();
        let mtu = buf.read_i16_be_bytes();
        let client_guid = buf.read_i64_be_bytes();

        Self {
            magic,
            server_address,
            mtu,
            client_guid,
        }
    }
}

pub struct OfflineConnRep2 {
    pub magic: [u8; 16],
    pub server_guid: i64,
    pub client_address: SocketAddr,
    pub mtu: i16,
    pub use_encryption: bool,
}

impl ToBuffer for OfflineConnRep2 {
    fn to_buffer(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_magic(&self.magic);
        buf.write_i64_be_bytes(&self.server_guid);
        buf.write_byte(self.use_encryption as u8);
        buf.write_i16_be_bytes(&self.mtu);

        buf
    }
}
