use super::obj::{FromBuffer, ToBuffer};
use crate::raknet::objects::MsgBuffer;

// TODO: acknack can have many bodies of records

fn write_body(input_records: &[u32]) -> MsgBuffer {
    // TODO: for some reason it sends this:
    // [192, 0, 1, 1, 0, 0, 0, 0, 0, 0]
    let mut acknack = MsgBuffer::new();
    let mut records = input_records.to_owned();
    records.sort();

    let mut section: Vec<u32> = vec![];
    for index in 0..records.len() {
        let current = records[index];
        section.push(current);
        if index + 1 < records.len() && records[index + 1] == current + 1 {
            continue;
        }

        acknack.write_i16_be_bytes(section.len() as i16);
        if section.len() > 1 {
            acknack.write_byte(0x00);
            acknack.write_u24_le_bytes(*section.first().unwrap());
        } else {
            acknack.write_byte(0x01);
            acknack.write_u24_le_bytes(*section.first().unwrap());
            acknack.write_u24_le_bytes(*section.last().unwrap());
        }
        section = vec![];
    }

    acknack
}

fn read_body(buf: &mut MsgBuffer) -> Vec<u32> {
    let _record_count = buf.read_i16_be_bytes();
    let mut records: Vec<u32> = vec![];

    loop {
        if buf.at_end() {
            break;
        }

        let is_range = buf.read_byte() != 1;

        if !is_range {
            let record = buf.read_u24_le_bytes();

            records.push(record);
        } else {
            let start_index = buf.read_u24_le_bytes();
            let end_index = buf.read_u24_le_bytes();

            let range: Vec<u32> = (start_index..=end_index).collect();
            records.extend_from_slice(&range);
        }
    }

    records
}

pub struct Ack {
    pub records: Vec<u32>,
}

impl ToBuffer for Ack {
    fn to_buffer(&self) -> MsgBuffer {
        write_body(&self.records)
    }
}

impl FromBuffer for Ack {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        Self {
            records: read_body(buf),
        }
    }
}

pub struct Nack {
    pub records: Vec<u32>,
}

impl ToBuffer for Nack {
    fn to_buffer(&self) -> MsgBuffer {
        write_body(&self.records)
    }
}

impl FromBuffer for Nack {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        Self {
            records: read_body(buf),
        }
    }
}
