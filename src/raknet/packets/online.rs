use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use super::{FromBuffer, ToBuffer};
use crate::raknet::objects::{
    datatypes::{get_unix_milis, to_address_bytes},
    MsgBuffer,
};

pub struct OnlineConnReq {
    pub guid: i64,
    pub timestamp: i64,
}

impl FromBuffer for OnlineConnReq {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        let guid = buf.read_i64_be_bytes();
        let timestamp = buf.read_i64_be_bytes();

        Self { guid, timestamp }
    }
}

pub struct OnlineConnAccepted {
    pub client_address: SocketAddr,
    // ignore system index
    // ignore internal IDs
    pub timestamp: i64,
}

impl ToBuffer for OnlineConnAccepted {
    fn to_buffer(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_address(&self.client_address);
        buf.write_i16_be_bytes(&0); // like, ok
        let mystery_address = to_address_bytes(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
            19132,
        ));
        for _ in 0..10 {
            buf.write(&mystery_address);
        }
        buf.write_i64_be_bytes(&self.timestamp);
        buf.write_i64_be_bytes(&(get_unix_milis() as i64));

        buf
    }
}
