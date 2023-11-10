use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::objects::MsgBuffer;
use super::objects::Frame;
use super::packets::{ACK, NACK};
use super::packets::{Deserialise, Serialize};

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
    frames_queue: Arc<Mutex<HashMap<u32, FrameSet>>>,
    resend_queue: Arc<Mutex<HashMap<u32, FrameSet>>>,
    missing_records: Arc<Mutex<Vec<u32>>>,
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
            frames_queue: Arc::new(Mutex::new(HashMap::new())),
            resend_queue: Arc::new(Mutex::new(HashMap::new())),
            missing_records: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn call_online_event(&mut self, frame: Frame) {

    }

    pub fn create_nack(&mut self, first: u32, until: u32) -> MsgBuffer {
        let records: Vec<u32> = (first..until).collect();

        self.missing_records.lock().unwrap().retain(|val| records.contains(val));

        NACK {records}.serialize()
    }

    pub fn create_ack(&mut self, first: u32, until: u32) -> MsgBuffer {
        let records: Vec<u32> = (first..until).collect();

        for rec in &records {
            self.missing_records.lock().unwrap().push(*rec);
        }

        ACK {records}.serialize()
    }

    pub async fn receive_frame_set(&mut self, frame_set: FrameSet) -> Vec<MsgBuffer> {
        let mut packets_to_send: Vec<MsgBuffer> = vec![];

        // First check for missing frames and sending NACKs
        if frame_set.index > self.client_frame_set_index+1 {
            packets_to_send.push(self.create_nack(self.client_frame_set_index+1, frame_set.index));
        }

        // As well as an ACK
        packets_to_send.push(self.create_ack(frame_set.index, frame_set.index+1));

        self.client_frame_set_index = frame_set.index;

        for frame in frame_set.frames {
            self.call_online_event();
        }

        // package everything currently

        packets_to_send
    }
}
