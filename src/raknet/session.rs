use std::collections::HashMap;
use std::net::SocketAddr;

use super::objects::MsgBuffer;
use super::objects::Frame;

pub struct FrameSet {
    pub index: u32,
    pub frames: Vec<Frame>,
}

pub struct Session {
    pub sockaddr: SocketAddr,
    guid: i64,
    mtu: i16,
    server_frame_set_index: u32,
    client_frame_set_index: u32,
    resend_queue: HashMap<u32, Frame>,
    missing_records: Vec<u32>,
}

impl Session {
    pub fn new(
        sockaddr: SocketAddr,
        client_guid: i64,
        mtu: i16,
    ) -> Self {
        Self {
            sockaddr: sockaddr,
            guid: client_guid,
            mtu: mtu,
            server_frame_set_index: 0,
            client_frame_set_index: 0,
            resend_queue: HashMap::new(),
            missing_records: vec![],
        }
    }

    pub fn acknack(&mut self, records: Vec<u32>, id: u8) -> MsgBuffer {
        // S->C Common code for writing the body of an ACK or NACK message TO client
        let mut acknack = MsgBuffer::new();
        acknack.write_byte(id);

        acknack.write_i16_be_bytes(&(records.len() as i16));
        if records.len() > 1 {
            acknack.write_byte(0x00);
            acknack.write_u24_le_bytes(&records.first().unwrap());  // shouldn't fail
        } else {
            acknack.write_byte(0x01);
            acknack.write_u24_le_bytes(&records.first().unwrap());  // shouldn't fail
            acknack.write_u24_le_bytes(&records.last().unwrap());  // shouldn't fail
        }

        acknack
    }

    pub fn nack(&mut self, first: u32, until: u32) -> MsgBuffer {
        // S->C Record not received
        // TODO: potentially redo
        let records: Vec<u32> = (first..until).collect();

        self.missing_records.retain(|val| records.contains(val));

        self.acknack(records, 0xa0)
    }

    pub fn ack(&mut self, first: u32, until: u32) -> MsgBuffer {
        // S->C Record received
        let records: Vec<u32> = (first..until).collect();

        for rec in &records {
            self.missing_records.push(*rec);
        }

        self.acknack(records, 0xc0)
    }

    pub fn receive_frame_set(&mut self, frame_set: FrameSet) -> Vec<MsgBuffer> {
        // some pseudocode
        let mut packets: Vec<MsgBuffer> = vec![];
        let frame_set_index = frame_set.index;

        // handle nacks
        if frame_set_index > self.client_frame_set_index+1 {
            packets.push(self.nack(self.client_frame_set_index+1, frame_set_index));
        }

        packets.push(self.ack(frame_set_index, frame_set_index+1));

        self.client_frame_set_index = frame_set_index;

        packets
    }
}
