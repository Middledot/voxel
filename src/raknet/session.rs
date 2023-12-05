use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use log::warn;
use tokio::sync::mpsc::Sender;

use crate::raknet::objects::datatypes::to_i32_varint_bytes;

use super::objects::datatypes::get_unix_milis;
use super::objects::msgbuffer::Packet;
use super::objects::{
    msgbuffer::{PacketPriority, SendPacket},
    MsgBuffer,
};
use super::objects::{FragmentInfo, Reliability};
use super::packets::*;
use super::packets::{Ack, Nack, OnlineConnAccepted, OnlineConnReq};
use super::packets::{FromBuffer, ToBuffer};

pub struct Session {
    pub sockaddr: SocketAddr,
    tx: Sender<(SendPacket, SocketAddr)>,
    pub guid: i64,
    pub server_guid: i64,
    pub mtu: i16,

    // tick: u64,
    // tick_interval: u64,
    fs_server_index: u32, // fs = frameset
    fs_client_index: u32,
    rel_client_index: u32,
    rel_server_index: u32,
    ord_channels: Vec<u32>,
    pub send_heap: BinaryHeap<SendPacket>,
    frames_queue: Arc<Mutex<BinaryHeap<Frame>>>,
    pub recv_queue: Vec<Packet>,
    pub send_queue: Vec<Packet>,
    resend_queue: Arc<Mutex<HashMap<u32, FrameSet>>>,
    missing_records: Arc<Mutex<Vec<u32>>>,
}

impl Session {
    pub fn new(
        sockaddr: SocketAddr,
        guid: i64,
        server_guid: i64,
        mtu: i16,
        tx: Sender<(SendPacket, SocketAddr)>,
    ) -> Self {
        Self {
            sockaddr,
            tx,
            guid,
            server_guid,
            mtu,
            fs_server_index: 0,
            fs_client_index: 0,
            rel_client_index: 0,
            rel_server_index: 0,
            ord_channels: vec![],
            send_heap: BinaryHeap::new(),
            frames_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            recv_queue: vec![],
            send_queue: vec![],
            resend_queue: Arc::new(Mutex::new(HashMap::new())),
            missing_records: Arc::new(Mutex::new(vec![])),
        }
    }

    fn next_fs_index(&mut self) -> u32 {
        self.fs_server_index += 1;

        self.fs_server_index - 1
    }

    pub async fn recv(&mut self, packet: Packet) {
        self.recv_queue.push(packet);
    }

    pub async fn tick(&mut self) {
        let packets = std::mem::take(&mut self.recv_queue);
        for packet in packets {
            match packet.packet_id {
                0xa0 => self.recv_nack(packet).await,
                0xc0 => self.recv_ack(packet).await,
                0x80..=0x8d => self.recv_frame_set(packet).await,
                _ => panic!("Packet ID {} is not implemented", packet.packet_id),
            };
        }

        // if self.send_heap.peek().unwrap().priority == PacketPriority::Immediate {
        //     return true;
        // }
        // return false;

        if self.send_heap.is_empty() {
            return;
        }

        // package into frame sets
        let mut frameset = FrameSet {
            index: self.next_fs_index(),
            frames: vec![],
        };
        let mut current_prio = PacketPriority::Medium;

        let mut frames_queue = self.frames_queue.lock().unwrap();
        let mut resend_queue = self.resend_queue.lock().unwrap();

        // frames_queue.sort_by_key(|x| {
        //     match x.reliability.rel_frameindex {
        //         Some(e) => e,
        //         None => 4
        //     }
        // });

        // TODO: add a system where it keeps adding frames based on prio
        // until a timer runs out and the next tick runs
        // TODO: actually we need to somehow mix in different prio types,
        // maybe need to switch data structures.

        for _ in 0..frames_queue.len() {
            let frame = match frames_queue.pop() {
                Some(pack) => pack,
                None => return,
            };
            current_prio = frame.priority.unwrap();

            if let Some(new_frameset) = frameset.try_add_frame(frame, self.mtu as u16) {
                self.send_heap.push(frameset.package(current_prio));
                resend_queue.insert(frameset.index, frameset);
                frameset = new_frameset;
                self.fs_server_index += 1; // I already increment in try_add_frame, just need to match here                
            }
        }

        self.send_heap.push(frameset.package(current_prio));
        resend_queue.insert(frameset.index, frameset);
    }

    fn send(&mut self, packet: SendPacket) {
        self.send_heap.push(packet);
    }

    async fn send_frame(&mut self, mut frame: Frame, priority: PacketPriority) {
        // immediate frames sent in new frames sets immediately
        // others are just added to the frames queue + other function to package them

        // example packet that works (from wireshark)
        // 0000   02 00 00 00 45 00 00 38 7a 44 00 00 80 11 00 00   ....E..8zD......
        // 0010   7f 00 00 01 7f 00 00 01 4a be ee 2c 00 24 e6 a3   ........J..,.$..
        // 0020   80 04 00 00 64 00 70 01 00 00 00 00 00 00 fe 0c   ....d.p.........
        // 0030   8f 01 01 00 00 00 00 00 00 00 00 00               ............
        if priority == PacketPriority::Immediate {
            let mut frameset = FrameSet {
                index: self.next_fs_index(),
                frames: vec![],
            };
            frameset.add_frame(frame);
            self.tx
                .send((frameset.package(priority), self.sockaddr))
                .await
                .unwrap_or_else(|_| warn!("Failed to send packet"));
        } else {
            frame.priority = Some(priority);
            self.frames_queue.lock().unwrap().push(frame);
        }
    }

    async fn send_default_frame(
        &mut self,
        packet_id: u8,
        body: MsgBuffer,
        priority: PacketPriority,
    ) {
        self.rel_server_index += 1;

        match self.ord_channels.get(0) {
            Some(_) => self.ord_channels[0] += 1,
            None => self.ord_channels.insert(0, 0),
        }

        // rust compiler my beloved
        let fs_index = self.next_fs_index();

        self.send_frame(
            Frame::from_default_options(packet_id, body, fs_index, self.ord_channels[0]),
            priority,
        )
        .await;
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

    pub fn send_ack(&mut self, first: u32, until: u32) {
        let records: Vec<u32> = (first..until).collect();

        for rec in &records {
            self.missing_records.lock().unwrap().push(*rec);
        }

        let buf = Ack { records }.to_buffer();
        self.send(SendPacket {
            packet_id: 0xc0,
            body: buf,
            priority: PacketPriority::Immediate,
        });
    }

    pub fn send_nack(&mut self, first: u32, until: u32) {
        let mut records: Vec<u32> = (first..until).collect();

        self.missing_records.lock().unwrap().append(&mut records);

        let buf = Nack { records }.to_buffer();
        self.send(SendPacket {
            packet_id: 0xa0,
            body: buf,
            priority: PacketPriority::Immediate,
        });
    }

    pub async fn recv_ping(&mut self, mut packet: Packet) {
        let mut pong = MsgBuffer::new();
        pong.write_i64_be_bytes(packet.body.read_i64_be_bytes());
        pong.write_i64_be_bytes(get_unix_milis() as i64);

        let frame = Frame {
            flags: 0,
            bitlength: (pong.len() * 8) as u16,
            bodysize: pong.len() as u16,
            reliability: Reliability::extract(0, &mut MsgBuffer::new()),
            fragment_info: FragmentInfo {
                is_fragmented: false,
                compound_size: None,
                compound_id: None,
                index: None,
            },
            inner_packet_id: 0x00,
            body: pong,
            priority: Some(PacketPriority::Immediate),
        };

        self.send_frame(frame, PacketPriority::Immediate).await;
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

        for frame in frameset.frames {
            self.adjust_internal(&frame);
            let packet = Packet {
                packet_id: frame.inner_packet_id,
                timestamp: packet.timestamp,
                body: frame.body,
            };

            match frame.inner_packet_id {
                0x00 => self.recv_ping(packet).await,
                0x13 => self.recv_frame_new_incoming_connection(packet).await,
                0x09 => self.recv_frame_connection_request(packet).await,
                0xfe => self.recv_game_packet(packet).await,
                0x15 => return, // TODO:
                _ => panic!("uh oh <:O {}", frame.inner_packet_id),
            };
        }
    }

    pub async fn recv_frame_connection_request(&mut self, mut packet: Packet) {
        let request = OnlineConnReq::from_buffer(&mut packet.body);

        OnlineConnAccepted {
            client_address: self.sockaddr,
            timestamp: request.timestamp,
        }
        .to_buffer();

        // self.rel_server_index += 1;
        // self.ord_channels[0] += 1;

        self.send_default_frame(
            0x10,
            OnlineConnAccepted {
                client_address: self.sockaddr,
                timestamp: request.timestamp,
            }
            .to_buffer(),
            PacketPriority::Medium,
        ).await;
    }

    pub async fn recv_frame_new_incoming_connection(&mut self, _packet: Packet) {
        // Technically don't even have to parse this
        // let _request = NewIncomingConnection::from_buffer(&mut packet.body);
        // println!("hier {:?}", request.internal_address);
    }

    pub async fn recv_game_packet(&mut self, mut packet: Packet) {
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
        // http://www.raknet.net/raknet/manual/systemoverview.html
        println!("lol");

        let mut game_packets = vec![];

        loop {
            if packet.body.at_end() {
                break;
            }

            let packetsize = packet.body.read_i32_varint_bytes() as usize;
            game_packets.push(packet.body.read_vec(packetsize));
        }

        let mut response: Vec<u8> = vec![];

        for packet in game_packets {
            let mut reader = MsgBuffer::from(packet);
            let firstunit = reader.read_i32_varint_bytes();
            let (_sub_client_id, _sub_sender_id, packet_id) = (
                (firstunit & 0x3000) >> 12,
                (firstunit & 0xc00) >> 10,
                firstunit & 0x3ff,
            );

            if packet_id == 0x01 {
                panic!("Ooooooooooooooooooooooo");
            }

            let mut resp = to_i32_varint_bytes(
                0_i32 | 0 /*sub_client_id << 12*/ | 0 /*sub_sender_id << 10*/ | 0x8F,
            );
            let mut bytes: Vec<u8> = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            resp.append(&mut bytes);
            // let mut resp: Vec<u8> = match packet[0] {
            //                         // 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0
            //     0xc1 => {
            //         // vec![12, 32, 143, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            //         let mut thing = MsgBuffer::from(packet)
            //     },
            //     _ => panic!("d"),
            // };
            // [-2, 12, 143, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            resp.insert(0, resp.len() as u8);
            println!("{:?}", &resp);
            response.append(&mut resp);
        }

        self.send_default_frame(0xfe, MsgBuffer::from(response), PacketPriority::Medium).await;
    }

    pub fn adjust_internal(&mut self, frame: &Frame) {
        // TODO: THIS ASSUMES THEY'RE SORTED
        // ARE YOU SURE THEY'RE SORTED?
        // I DON'T THINK YOU'RE SURE THEY'RE SORTED

        if let Some(rel_frameindex) = frame.reliability.rel_frameindex {
            if self.rel_client_index < rel_frameindex {
                self.rel_client_index = rel_frameindex;
            }
        }

        // TODO: sequenced stuff

        if frame.reliability.is_ordered() {
            let ord_channel = frame.reliability.ord_channel.unwrap();
            let ord_frameindex = frame.reliability.ord_frameindex.unwrap();

            if self.ord_channels.get(ord_channel as usize).unwrap() < &ord_frameindex {
                self.ord_channels[ord_channel as usize] = ord_frameindex;
            }

            // if self.ord_channels.contains_key(&ord_channel) {
            //     if self.ord_channels[&ord_channel] < ord_frameindex {
            //         *self.ord_channels.get_mut(&ord_channel).unwrap() = ord_frameindex;
            //     }
            // } else {
            //     self.ord_channels.insert(ord_channel, ord_frameindex);
            // }
        }
    }
}
