use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::objects::Frame;
use super::objects::MsgBuffer;
use super::objects::msgbuffer::Packet;
use super::objects::datatypes::get_unix_milis;
use super::packets::{Ack, Nack, OnlineConnAccepted, OnlineConnReq};
use super::packets::{FromBuffer, ToBuffer};
use super::packets::*;

pub struct FrameSet {
    pub index: u32,
    pub frames: Vec<Frame>,
}

impl FrameSet {
    pub fn currentsize(&self) -> u16 {
        self.frames.iter().map(|f| f.totalsize()).sum::<u16>() + 4
    }

    pub fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }
}

impl FromBuffer for FrameSet {
    fn from_buffer(buf: &mut MsgBuffer) -> Self {
        let index = buf.read_u24_le_bytes();
        let mut frames: Vec<Frame> = vec![];

        while !buf.at_end() {
            frames.push(
                Frame::from_buffer(buf)
            )
        }

        Self {
            index,
            frames,
        }
    }
}

impl ToBuffer for FrameSet {
    fn to_buffer(&self) -> MsgBuffer {
        let mut buf = MsgBuffer::new();
        buf.write_u24_le_bytes(&self.index);

        for fr in self.frames.iter() {
            buf.write_buffer(fr.to_buffer().get_bytes());
        }

        buf
    }
}

pub struct Session {
    pub sockaddr: SocketAddr,
    pub guid: i64,
    pub server_guid: i64,
    pub mtu: i16,
    server_frameset_index: u32,
    client_frameset_index: u32,
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
            server_frameset_index: 0,
            client_frameset_index: 0,
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

        self.server_frameset_index += 1;

        let mut frameset = FrameSet {
            index: self.server_frameset_index,
            frames: vec![],
        };

        let mut frames_queue = self.frames_queue.lock().unwrap();

        for _ in 0..frames_queue.len() {
            let frame = frames_queue.remove(0);
            if frameset.currentsize() + frame.totalsize() > self.mtu as u16 {
                self.send_queue.push(
                    Packet {
                        packet_id: 0x84,
                        timestamp: get_unix_milis(),
                        body: frameset.to_buffer(),
                    }
                );
                self.resend_queue.lock().unwrap().insert(frameset.index, frameset);

                self.server_frameset_index += 1;
                frameset = FrameSet {
                    index: self.server_frameset_index,
                    frames: vec![],
                };
            }

            frameset.add_frame(frame);
        }

        std::mem::replace(&mut self.send_queue, vec![])
    }

    pub async fn call_event(&mut self, packet: Packet) {
        match packet.packet_id {
            0x07 => self.recv_offline_connection_request_2(packet).await,
            0xa0 => self.recv_nack(packet).await,
            0xc0 => self.recv_ack(packet).await,
            0x80..=0x8d => self.recv_frame_set(packet).await,
            _ => panic!("Nous pouvons rien faire | There's nothing we can do ({})", packet.packet_id),
        }
    }

    pub async fn recv_ack(&mut self, mut packet: Packet) {
        let ack_pack = Ack::from_buffer(&mut packet.body);
        let mut resend_queue = self.resend_queue.lock().unwrap();

        for rec in ack_pack.records {
            resend_queue.remove(&rec);
        }
    }

    pub async fn recv_nack(&mut self, mut packet: Packet) {
        let nack_pack = Nack::from_buffer(&mut packet.body);
        let mut resend_queue = self.resend_queue.lock().unwrap();

        for rec in nack_pack.records {
            let frame_set = resend_queue.get_mut(&rec).unwrap();
            let packet = Packet {packet_id: 0x84, timestamp: packet.timestamp, body: frame_set.to_buffer()};

            self.send_queue.push(packet);
        }
    }

    pub async fn recv_offline_connection_request_2(&mut self, mut packet: Packet) {
        let request2 = OfflineConnReq2::from_buffer(&mut packet.body);

        let reply2 = OfflineConnRep2 {
            magic: request2.magic,
            server_guid: self.server_guid,
            client_address: self.sockaddr,
            mtu: self.mtu as i16,
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

    pub async fn recv_frame_set(&mut self, mut packet: Packet) {
        // TODO: wtf is this:
        // [132, 0, 0, 0, 64, 2, 248, 0, 95, 0, 0, 0, 4, 127, 0, 0, 1, 235, 158, 0, 0, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 4, 255, 255, 255, 255, 74, 188, 0, 0, 0, 2, 112, 138, 54, 252, 0, 0, 1, 139, 198, 176, 186, 127]
        // also [132, 0, 0, 0, 64, 0, 144, 0, 0, 0, 9, 131, 237, 153, 211, 18, 169, 106, 213, 0, 0, 0, 2, 56, 60, 233, 205, 0]

        let frameset = FrameSet::from_buffer(&mut packet.body);

        if frameset.index > self.client_frameset_index + 1 {
            self.send_nack(self.client_frameset_index + 1, frameset.index);
        }
        self.send_ack(frameset.index, frameset.index + 1);

        self.client_frameset_index = frameset.index;
        let mut frames_tosend: Vec<Frame> = vec![];
        
        // = self.frames_queue
        //    .lock()
        //    .unwrap();

        for frame in frameset.frames {
            let packet = Packet {
                packet_id: frame.inner_packet_id,
                timestamp: packet.timestamp,
                body: frame.body,
            };

            let mut reply = match frame.inner_packet_id {
                0x09 => self.recv_frame_connection_request(packet).await,
                _ => panic!("oh no"),
            };

            frames_tosend.push(
                Frame {
                    flags: frame.flags,
                    bitlength: (reply.len() * 8) as u16,
                    bodysize: reply.len() as u16,
                    reliability: frame.reliability,
                    fragment_info: frame.fragment_info,
                    inner_packet_id: 0x10,
                    body: reply,
                }
            );
        }

        self.frames_queue.lock().unwrap().append(&mut frames_tosend);
    }

    pub async fn recv_frame_connection_request(&mut self, mut packet: Packet) -> MsgBuffer {
        let request = OnlineConnReq::from_buffer(&mut packet.body);

        OnlineConnAccepted {
            client_address: self.sockaddr,
            timestamp: request.timestamp,
        }.to_buffer()
    }

    pub fn send_nack(&mut self, first: u32, until: u32) {
        let mut records: Vec<u32> = (first..until).collect();

        self.missing_records
            .lock()
            .unwrap()
            .append(&mut records);

        let buf = Nack { records }.to_buffer();
        self.send_queue.push(
            Packet {
                packet_id: 0xa0,
                timestamp: get_unix_milis(),
                body: buf
            }
        );
    }

    pub fn send_ack(&mut self, first: u32, until: u32) {
        let records: Vec<u32> = (first..until).collect();

        for rec in &records {
            self.missing_records.lock().unwrap().push(*rec);
        }

        let buf = Ack { records }.to_buffer();
        self.send_queue.push(
            Packet {
                packet_id: 0xc0,
                timestamp: get_unix_milis(),
                body: buf
            }
        );
    }

    // pub fn create_frame(&mut self, old_frame: Frame, packet: Packet) {

    // }
}
