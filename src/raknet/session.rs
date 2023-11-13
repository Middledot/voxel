use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::objects::Frame;
use super::objects::MsgBuffer;
use super::packets::{Ack, Nack, OnlineConnAccepted, OnlineConnReq};
use super::packets::{FromBuffer, ToBuffer};

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
    frames_queue: Arc<Mutex<Vec<MsgBuffer>>>,
    resend_queue: Arc<Mutex<HashMap<u32, FrameSet>>>,
    missing_records: Arc<Mutex<Vec<u32>>>,
}

impl Session {
    pub fn new(sockaddr: SocketAddr, client_guid: i64, mtu: i16) -> Self {
        Self {
            sockaddr,
            guid: client_guid,
            mtu,
            server_frame_set_index: 0,
            client_frame_set_index: 0,
            frames_queue: Arc::new(Mutex::new(vec![])),
            resend_queue: Arc::new(Mutex::new(HashMap::new())),
            missing_records: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn online_conn_req(&mut self, mut frame: Frame) {
        let conn_req = OnlineConnReq::from_buffer(&mut frame.body);

        let mut conn_accept = OnlineConnAccepted {
            client_address: self.sockaddr,
            timestamp: conn_req.timestamp,
        }
        .to_buffer();

        let mut respframe = Frame {
            flags: frame.flags,
            bitlength: (conn_accept.len() * 8) as u16,
            bodysize: conn_accept.len() as u16,
            reliability: frame.reliability,
            fragment_info: frame.fragment_info,
            inner_packet_id: 0x10,
            body: conn_accept,
        };

        self.frames_queue
            .lock()
            .unwrap()
            .push(respframe.serialize())
    }

    pub async fn call_online_event(&mut self, frame: Frame) {
        match frame.inner_packet_id {
            0x09 => self.online_conn_req(frame).await,
            _ => panic!("It's joever | C'est joever"),
        }
    }

    pub fn create_nack(&mut self, first: u32, until: u32) -> MsgBuffer {
        let records: Vec<u32> = (first..until).collect();

        self.missing_records
            .lock()
            .unwrap()
            .retain(|val| records.contains(val));

        Nack { records }.to_buffer()
    }

    pub fn create_ack(&mut self, first: u32, until: u32) -> MsgBuffer {
        let records: Vec<u32> = (first..until).collect();

        for rec in &records {
            self.missing_records.lock().unwrap().push(*rec);
        }

        Ack { records }.to_buffer()
    }

    pub fn recv_ack(&mut self, _records: Ack) {
        // TODO
    }

    pub async fn recv_frame_set(&mut self, frame_set: FrameSet) -> Vec<MsgBuffer> {
        let mut packets_to_send: Vec<MsgBuffer> = vec![];

        // First check for missing frames and sending Nacks
        if frame_set.index > self.client_frame_set_index + 1 {
            packets_to_send
                .push(self.create_nack(self.client_frame_set_index + 1, frame_set.index));
        }

        // As well as an Ack
        packets_to_send.push(self.create_ack(frame_set.index, frame_set.index + 1));

        self.client_frame_set_index = frame_set.index;

        for frame in frame_set.frames {
            self.call_online_event(frame).await;
        }

        // package everything currently

        let mut frames_queue = self.frames_queue.lock().unwrap();
        for _ in 0..frames_queue.len() {
            packets_to_send.push(frames_queue.remove(0));
        }

        packets_to_send
    }
}
