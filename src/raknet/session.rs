use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::raknet::objects::datatypes::to_i32_varint_bytes;

use super::objects::datatypes::get_unix_milis;
use super::objects::msgbuffer::Packet;
use super::objects::reliability::ReliabilityType;
use super::objects::{MsgBuffer, Reliability};
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
                0xa0 => self.recv_nack(packet).await,
                0xc0 => self.recv_ack(packet).await,
                0x80..=0x8d => self.recv_frame_set(packet).await,
                _ => panic!("Packet ID {} is not implemented", packet.packet_id),
            };
        }

        // package into frame sets
        let mut frameset = FrameSet {
            index: self.fs_server_index,
            frames: vec![],
        };

        let mut frames_queue = self.frames_queue.lock().unwrap();

        for i in 0..frames_queue.len() {
            let frame = frames_queue.remove(0);
            if i > 0 {//frameset.currentsize() + frame.totalsize() > self.mtu as u16 {
                self.send_queue.push(Packet {
                    packet_id: 0x84,
                    timestamp: get_unix_milis(),
                    body: frameset.to_buffer(),
                });
                self.resend_queue
                    .lock()
                    .unwrap()
                    .insert(frameset.index, frameset);
                frameset = FrameSet {
                    index: self.fs_server_index + 1,
                    frames: vec![],
                };
            }

            frameset.add_frame(frame);
            if frameset.frames.len() == 1 && i> 0 {
                self.fs_server_index += 1;
            }
            println!("{}", self.fs_server_index);
        }

        if frameset.frames.len() > 0 {
            self.send_queue.push(Packet {
                packet_id: 0x84,
                timestamp: get_unix_milis(),
                body: frameset.to_buffer(),
            })
        }
    }

    // pub async fn call_event(&mut self, packet: Packet) -> Option<MsgBuffer> {
    //     match packet.packet_id {
    //         0x09 => Some(
    //             self.recv_frame_connection_request(packet).await
    //         ),
    //         0x13 => {
    //             self.recv_frame_new_incoming_connection(packet).await;
    //             None
    //         },
    //         0xfe => {
    //             self.game_packet(packet).await;
    //         }
    //         _ => panic!(
    //             "Nous pouvons rien faire | There's nothing we can do ({}) {:?}",
    //             packet.packet_id,
    //             packet.body
    //         ),
    //     }
    // }

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

    pub async fn recv_ping(&self, mut packet: Packet) -> MsgBuffer {
        let mut pong = MsgBuffer::new();
        pong.write_i64_be_bytes(packet.body.read_i64_be_bytes());
        pong.write_i64_be_bytes(get_unix_milis() as i64);

        pong
    }

    pub async fn recv_frame_set(&mut self, mut packet: Packet) {
        let frameset = FrameSet::from_buffer(&mut packet.body);

        if frameset.frames.get(0).unwrap().reliability.is_reliable() {
            if frameset.index > self.fs_client_index + 1 {
                self.send_nack(self.fs_client_index + 1, frameset.index);
            }
            self.send_ack(frameset.index, frameset.index + 1);
        }

        self.fs_client_index = frameset.index;
        let mut frames_to_send: Vec<Frame> = vec![];
        // let t = frameset.frames.len();

        for mut frame in frameset.frames {
            let packet = Packet {
                packet_id: frame.inner_packet_id,
                timestamp: packet.timestamp,
                body: frame.body,
            };

            // handle no-reply packets
            match frame.inner_packet_id {
                0x13 => {
                    self.recv_frame_new_incoming_connection(packet).await;
                    continue;
                },
                _ => {}
            };

            let (packet_id, mut reply) = match frame.inner_packet_id {
                0x00 => (0x03, self.recv_ping(packet).await),
                0x09 => (0x10, self.recv_frame_connection_request(packet).await),
                0xfe => (0xfe, self.recv_game_packet(packet).await),
                0x15 => return,
                _ => panic!("uh oh <:O {}", frame.inner_packet_id)
            };

            if packet_id == 0xfe {
                // frame.reliability = Reliability {
                //     reltype: ReliabilityType::from_flags(64),
                //     rel_frameindex: Some(2),
                //     seq_frameindex: None,
                //     ord_frameindex: None,
                //     ord_channel: None,
                // };
                frame.reliability.ord_frameindex = Some(frame.reliability.ord_frameindex.unwrap() - 1);
                let fr = Frame {
                    flags: frame.flags,
                    bitlength: (reply.len() * 8) as u16,
                    bodysize: reply.len() as u16,
                    reliability: frame.reliability,
                    fragment_info: frame.fragment_info,
                    inner_packet_id: packet_id,
                    body: reply,
                };
                frames_to_send.push(fr);
                // let fr2 = Frame {
                //     flags: frame.flags,
                //     bitlength: (reply.len() * 8) as u16,
                //     bodysize: reply.len() as u16,
                //     reliability: frame.reliability,
                //     fragment_info: frame.fragment_info,
                //     inner_packet_id: packet_id,
                //     body: reply,
                // };
                // frames_to_send.push(fr2);
                continue;
            }

            frames_to_send.push(Frame {
                flags: frame.flags,
                bitlength: (reply.len() * 8) as u16,
                bodysize: reply.len() as u16,
                reliability: frame.reliability,
                fragment_info: frame.fragment_info,
                inner_packet_id: packet_id,
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
        let _request = NewIncomingConnection::from_buffer(&mut packet.body);
        // println!("hier {:?}", request.internal_address);
    }

    pub async fn recv_game_packet(&mut self, mut packet: Packet) -> MsgBuffer {
        // istg the api is nested as heck
        // frameset{
        //    frame{
        //        gamepacket{
        //            a game packet,
        //            ...
        //        }
        //    },
        //    ...
        // }
        println!("lol");

        let mut game_packets = vec![];

        loop {
            if packet.body.at_end() {
                break;
            }

            let packetsize = packet.body.read_i32_varint_bytes() as usize;
            game_packets.push(
                packet.body.read_vec(packetsize)
            );
        }

        let mut response: Vec<u8> = vec![];

        for packet in game_packets {
            let mut reader = MsgBuffer::from(packet);
            let firstunit = reader.read_i32_varint_bytes();
            let (sub_client_id, sub_sender_id, packet_id) = (
                (firstunit & 0x3000) >> 12,
                (firstunit & 0xc00) >> 10,
                firstunit & 0x3ff,
            );

            let mut resp = to_i32_varint_bytes(0_i32 | sub_client_id << 12 | sub_sender_id << 10 | 0x8F);
            let mut bytes: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            resp.append(&mut bytes);
            // let mut resp: Vec<u8> = match packet[0] {
            //                         // 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0
            //     0xc1 => {
            //         // vec![12, 32, 143, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            //         let mut thing = MsgBuffer::from(packet)
            //     },
            //     _ => panic!("d"),
            // };
            resp.insert(0, resp.len() as u8);
            println!("{:?}", &resp);
            response.append(&mut resp);
        }

        MsgBuffer::from(response)
    }

}
