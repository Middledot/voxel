use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::objects::Frame;
use super::objects::MsgBuffer;
use super::objects::msgbuffer::Packet;
use super::packets::{Ack, Nack, OnlineConnAccepted, OnlineConnReq};
use super::packets::{FromBuffer, ToBuffer};
use super::packets::*;

pub struct FrameSet {
    pub index: u32,
    pub frames: Vec<Frame>,
}

impl FrameSet {
    fn to_buffer(&mut self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_u24_le_bytes(&self.index);

        for fr in self.frames.iter_mut() {
            buf.write_buffer(&mut fr.serialize());
        }

        buf
    }
}

pub struct Session {
    pub sockaddr: SocketAddr,
    pub guid: i64,
    pub server_guid: i64,
    pub mtu: i16,
    server_frame_set_index: u32,
    client_frame_set_index: u32,
    pub recv_queue: Vec<Packet>,
    pub send_queue: Vec<Packet>,
    frames_queue: Arc<Mutex<Vec<Frame>>>,
    resend_queue: Arc<Mutex<HashMap<u32, FrameSet>>>,
    missing_records: Arc<Mutex<Vec<u32>>>,
}

impl Session {
    pub fn new(sockaddr: SocketAddr, guid: i64, server_guid: i64, mtu: i16) -> Self {
        Self {
            sockaddr,
            guid,
            server_guid,
            mtu,
            server_frame_set_index: 0,
            client_frame_set_index: 0,
            recv_queue: vec![],
            send_queue: vec![],
            frames_queue: Arc::new(Mutex::new(vec![])),
            resend_queue: Arc::new(Mutex::new(HashMap::new())),
            missing_records: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn recv(&mut self, packet: Packet) {
        self.recv_queue.push(packet);
    }

    pub async fn update(&mut self) -> Vec<Packet> {
        let packets = std::mem::replace(&mut self.recv_queue, vec![]);
        for packet in packets {
            self.call_event(packet).await;
        }

        std::mem::replace(&mut self.send_queue, vec![])
    }

    pub async fn call_event(&mut self, packet: Packet) {
        match packet.packet_id {
            0x07 => self.recv_offline_connection_request_2(packet),
            0xa0 => self.recv_nack(packet),
            0xc0 => self.recv_ack(packet),
            _ => panic!("Nous pouvons rien faire | There's nothing we can do"),
        }
    }

    pub fn recv_ack(&mut self, mut packet: Packet) {
        let ack_pack = Ack::from_buffer(&mut packet.body);
        let mut resend_queue = self.resend_queue.lock().unwrap();

        for rec in ack_pack.records {
            resend_queue.remove(&rec);
        }
    }

    pub fn recv_nack(&mut self, mut packet: Packet) {
        let nack_pack = Nack::from_buffer(&mut packet.body);
        let mut resend_queue = self.resend_queue.lock().unwrap();

        for rec in nack_pack.records {
            let frame_set = resend_queue.get_mut(&rec).unwrap();
            let packet = Packet {packet_id: 0x84, timestamp: packet.timestamp, body: frame_set.to_buffer()};

            self.send_queue.push(packet);
        }
    }

    pub fn recv_offline_connection_request_2(&mut self, mut packet: Packet) {
        let request2 = OfflineConnReq2::from_buffer(&mut packet.body);

        let reply2 = OfflineConnRep2 {
            magic: request2.magic,
            server_guid: self.server_guid,
            client_address: self.sockaddr,
            mtu: self.mtu,
            use_encryption: false, // disable encryption // TODO: look into? what is this?
        };

        self.guid = request2.client_guid;

        self.send_queue.push(
            Packet {
                packet_id: 0x08,
                timestamp: packet.timestamp,
                body: reply2.to_buffer()
            }
        );
    }

    pub async fn online_conn_req(&mut self, mut frame: Frame) {
        let conn_req = OnlineConnReq::from_buffer(&mut frame.body);

        let mut conn_accept = OnlineConnAccepted {
            client_address: self.sockaddr,
            timestamp: conn_req.timestamp,
        }
        .to_buffer();

        let respframe = Frame {
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
            .push(respframe)
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
            // self.call_online_event(frame).await;
        }

        // package everything currently

        let mut frames_queue = self.frames_queue.lock().unwrap();
        self.server_frame_set_index += 1;

        let mut new_frame_set = FrameSet {index: self.server_frame_set_index, frames: vec![]};
        for _ in 0..frames_queue.len() {
            new_frame_set.frames.push(frames_queue.remove(0));
        }

        // TODO: wtf is this:
        // [132, 0, 0, 0, 64, 2, 248, 0, 95, 0, 0, 0, 4, 127, 0, 0, 1, 235, 158, 0, 0, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 0, 0, 0, 2, 112, 138, 54, 252, 0, 0, 1, 139, 198, 176, 186, 127]

        packets_to_send.push(new_frame_set.to_buffer());
        packets_to_send
    }
}
