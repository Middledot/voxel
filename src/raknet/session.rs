use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use super::objects::datatypes::get_unix_milis;
use super::objects::msgbuffer::Packet;
use super::objects::MsgBuffer;
use super::packets::*;
use super::packets::{Ack, Nack, OnlineConnAccepted, OnlineConnReq};
use super::packets::{FromBuffer, ToBuffer};

pub struct Session {
    pub sockaddr: SocketAddr,
    pub guid: i64,
    pub server_guid: i64,
    pub mtu: i16,
    fs_server_index: u32,  // imma use abbr here cuz 
    fs_client_index: u32,
    rel_client_index: u32,
    rel_server_index: u32,
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
            fs_server_index: 0,
            fs_client_index: 0,
            rel_client_index: 0,
            rel_server_index: 0,
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

    pub async fn update(&mut self) {
        let packets = std::mem::take(&mut self.recv_queue);
        for packet in packets {
            match packet.packet_id {
                0x80..=0x8d => {
                    self.recv_frame_set(packet).await;
                    None
                },
                _ => self.call_event(packet).await,
            };
        }

        self.fs_server_index += 1;

        let mut frameset = FrameSet {
            index: self.fs_server_index,
            frames: vec![],
        };

        let mut frames_queue = self.frames_queue.lock().unwrap();

        for _ in 0..frames_queue.len() {
            let frame = frames_queue.remove(0);
            if frameset.currentsize() + frame.totalsize() > self.mtu as u16 {
                self.send_queue.push(Packet {
                    packet_id: 0x84,
                    timestamp: get_unix_milis(),
                    body: frameset.to_buffer(),
                });
                self.resend_queue
                    .lock()
                    .unwrap()
                    .insert(frameset.index, frameset);

                self.fs_server_index += 1;
                frameset = FrameSet {
                    index: self.fs_server_index,
                    frames: vec![],
                };
            }

            frameset.add_frame(frame);
        }

        if frameset.frames.len() > 0 {
            self.send_queue.push(Packet {
                packet_id: 0x84,
                timestamp: get_unix_milis(),
                body: frameset.to_buffer(),
            })
        } else {
            self.fs_server_index -= 1;
        }
    }

    pub async fn call_event(&mut self, packet: Packet) -> Option<MsgBuffer> {
        match packet.packet_id {
            0xa0 => {
                self.recv_nack(packet).await;
                None
            },
            0xc0 => {
                self.recv_ack(packet).await;
                None
            },
            0x09 => Some(
                self.recv_frame_connection_request(packet).await
            ),
            0x10 => {
                self.recv_frame_new_incoming_connection(packet).await;
                None
            },
            _ => panic!(
                "Nous pouvons rien faire | There's nothing we can do ({})",
                packet.packet_id
            ),
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
            let packet = Packet {
                packet_id: 0x84,
                timestamp: packet.timestamp,
                body: frame_set.to_buffer(),
            };

            self.send_queue.push(packet);
        }
    }

    pub async fn recv_frame_set(&mut self, mut packet: Packet) {
        let frameset = FrameSet::from_buffer(&mut packet.body);

        if frameset.index > self.fs_client_index + 1 {
            self.send_nack(self.fs_client_index + 1, frameset.index);
        }
        self.send_ack(frameset.index, frameset.index + 1);

        self.fs_client_index = frameset.index;
        let mut frames_to_send: Vec<Frame> = vec![];

        for frame in frameset.frames {
            let packet = Packet {
                packet_id: frame.inner_packet_id,
                timestamp: packet.timestamp,
                body: frame.body,
            };

            let mut reply = match self.call_event(packet).await {
                Some(r) => r,
                None => continue,
            };

            frames_to_send.push(Frame {
                flags: frame.flags,
                bitlength: (reply.len() * 8) as u16,
                bodysize: reply.len() as u16,
                reliability: frame.reliability,
                fragment_info: frame.fragment_info,
                inner_packet_id: 0x10,
                body: reply,
            });
        }

        self.frames_queue.lock().unwrap().append(&mut frames_to_send);
    }

    pub async fn recv_frame_connection_request(&mut self, mut packet: Packet) -> MsgBuffer {
        let request = OnlineConnReq::from_buffer(&mut packet.body);

        OnlineConnAccepted {
            client_address: self.sockaddr,
            timestamp: request.timestamp,
        }
        .to_buffer()
    }

    pub async fn recv_frame_new_incoming_connection(&mut self, mut packet: Packet) {
        let request = NewIncomingConnection::from_buffer(&mut packet.body);
        println!("bollocks {:?}", request.internal_address);
    }

    pub fn send_nack(&mut self, first: u32, until: u32) {
        let mut records: Vec<u32> = (first..until).collect();

        self.missing_records.lock().unwrap().append(&mut records);

        let buf = Nack { records }.to_buffer();
        self.send_queue.push(Packet {
            packet_id: 0xa0,
            timestamp: get_unix_milis(),
            body: buf,
        });
    }

    pub fn send_ack(&mut self, first: u32, until: u32) {
        let records: Vec<u32> = (first..until).collect();

        for rec in &records {
            self.missing_records.lock().unwrap().push(*rec);
        }

        let buf = Ack { records }.to_buffer();
        self.send_queue.push(Packet {
            packet_id: 0xc0,
            timestamp: get_unix_milis(),
            body: buf,
        });
    }

    // pub fn create_frame(&mut self, old_frame: Frame, packet: Packet) {

    // }
}
