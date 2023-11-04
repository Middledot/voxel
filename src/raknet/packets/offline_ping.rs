use crate::raknet::objects::MsgBuffer;

use super::obj::{Deserialise, Serialize};

pub struct OfflinePing {
    pub timestamp: i64,
    pub magic: [u8; 16],
    pub client_guid: i64,
}

impl Deserialise for OfflinePing {
    const ID: u8 = 0x01;
    // for 0x02, we could either make an alias class, or ignore it completely

    fn deserialise(buf: &mut MsgBuffer) -> Self {
        // keep client_timestamp instead of timestamp the
        // distinction might be important in the future idk
        let client_timestamp = buf.read_i64_be_bytes();
        let magic = buf.read_magic();
        let client_guid = buf.read_i64_be_bytes();

        Self {
            timestamp: client_timestamp,
            magic: magic,
            client_guid: client_guid,
        }
    }
}

pub struct OfflinePong {
    pub timestamp: i64,
    pub server_guid: i64,
    pub magic: [u8; 16],
    pub server_name: String,
}

impl Serialize for OfflinePong {
    const ID: u8 = 0x1c;

    fn serialize(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_i64_be_bytes(&self.timestamp);
        buf.write_i64_be_bytes(&self.server_guid);
        buf.write_magic(&self.magic);
        buf.write_string(&self.server_name);

        buf
    }
}
