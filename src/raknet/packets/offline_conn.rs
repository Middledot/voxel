use std::net::SocketAddr;

use crate::raknet::objects::MsgBuffer;

use super::obj::{Serialize, Deserialise};

pub struct OfflineConnReq1 {
    pub magic: [u8; 16],
    pub protocol: u8, // mysterious magical mystical value, unknown use (always 0x11)
    pub mtu: i16,
}

impl Deserialise for OfflineConnReq1 {
    const ID: u8 = 0x05;

    fn deserialise(buf: &mut MsgBuffer) -> Self {
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

impl Serialize for OfflineConnRep1 {
    const ID: u8 = 0x06;

    fn serialize(&self) -> MsgBuffer {
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

impl Deserialise for OfflineConnReq2 {
    const ID: u8 = 0x07;

    fn deserialise(buf: &mut MsgBuffer) -> Self {
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

impl Serialize for OfflineConnRep2 {
    const ID: u8 = 0x08;

    fn serialize(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_magic(&self.magic);
        buf.write_i64_be_bytes(&self.server_guid);
        buf.write_byte(self.use_encryption as u8);
        buf.write_i16_be_bytes(&self.mtu);

        buf
    }
}