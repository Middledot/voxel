use super::obj::{Deserialise, Serialize};
use crate::raknet::objects::MsgBuffer;

// TODO: acknack can have many bodies of records

fn write_body(records: &Vec<u32>, id: u8) -> MsgBuffer {
    let mut acknack = MsgBuffer::new();
    acknack.write_byte(id);

    acknack.write_i16_be_bytes(&(records.len() as i16));
    if records.len() > 1 {
        acknack.write_byte(0x00);
        acknack.write_u24_le_bytes(records.first().unwrap()); // shouldn't fail
    } else {
        acknack.write_byte(0x01);
        acknack.write_u24_le_bytes(records.first().unwrap()); // shouldn't fail
        acknack.write_u24_le_bytes(records.last().unwrap()); // shouldn't fail
    }

    acknack
}

fn read_body(buf: &mut MsgBuffer) -> Vec<u32> {
    buf.read_byte();
    buf.read_i16_be_bytes();

    let is_range = buf.read_byte() != 0;

    if !is_range {
        buf.read_byte();
        let record = buf.read_u24_le_bytes();

        vec![record]
    } else {
        buf.read_byte();
        let start_index = buf.read_u24_le_bytes();
        let end_index = buf.read_u24_le_bytes();

        (start_index..=end_index).collect()
    }
}

pub struct Ack {
    pub records: Vec<u32>,
}

impl Serialize for Ack {
    fn serialize(&self) -> MsgBuffer {
        write_body(&self.records, 0xc0)
    }
}

impl Deserialise for Ack {
    fn deserialise(buf: &mut MsgBuffer) -> Self {
        Self {
            records: read_body(buf),
        }
    }
}

pub struct Nack {
    pub records: Vec<u32>,
}

impl Serialize for Nack {
    fn serialize(&self) -> MsgBuffer {
        write_body(&self.records, 0xa0)
    }
}

impl Deserialise for Nack {
    fn deserialise(buf: &mut MsgBuffer) -> Self {
        Self {
            records: read_body(buf),
        }
    }
}
