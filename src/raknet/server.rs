/// raknet/server.rs
/// ================
///
/// The server, one who handles RakNet packets.
///
/// Reference: https://wiki.vg/Raknet_Protocol
use rand::Rng;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::io::Error;
use std::{thread, time::{SystemTime, UNIX_EPOCH}};
use tokio::net::UdpSocket;

use log::trace;

use super::objects::msgbuffer::Packet;
use super::objects::{Frame, MsgBuffer};
use super::packets::*;
use super::session::{FrameSet, Session};
use crate::config::Config;


struct Socket {
    udpsock: UdpSocket,
}

impl Socket {
    pub async fn bind(addr: String) -> Self {
        Self {
            udpsock: UdpSocket::bind(addr).await.unwrap()  // might fail lol
        }
    }

    pub async fn send_packet(&self, packet_id: u8, packet: &mut MsgBuffer, client: SocketAddr) {
        let serialized = packet;
        let body = serialized.get_bytes();
        let mut bytes = vec![packet_id];
        bytes.extend_from_slice(body);

        self.send_to(&bytes, client).await;

        if !packet_id == 0x1c {
            trace!("0x{packet_id} SENT = {body:?}");
        }
    }

    pub async fn send_to(&self, buf: &[u8], target: SocketAddr) {
        self.udpsock.send_to(buf, target).await.expect(format!("Failed to send packet to {}", target.to_string()).as_str());
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        self.udpsock.recv_from(buf).await
    }
}


pub struct RakNetServer {
    socket: Socket,
    server_guid: i64,
    config: Config,
    sessions: HashMap<String, Session>,
    buf: [u8; 2048],
}

impl RakNetServer {
    pub async fn new(config: Config) -> Self {
        let socket = Socket::bind("127.0.0.1:".to_string() + config.get_property("server-port")).await;

        Self {
            socket,
            server_guid: rand::thread_rng().gen_range(1..=i64::MAX),
            config,
            sessions: HashMap::new(),
            buf: [0u8; 2048]
        }
    }

    pub fn get_server_name(&mut self) -> String {
        let motd = self.config.get_property("server-name");

        // so picky I don't get it smh
        vec![
            "MCPE",
            motd,
            "622",
            "1.20.40",
            self.sessions.len().to_string().as_str(),
            self.config.get_property("max-players").as_str(),
            self.server_guid.to_string().as_str(),
            motd,
            "Creative",
            "1",
            self.config.get_property("server-port").as_str(),
            self.config.get_property("server-portv6").as_str(),
        ]
        .join(";")
    }

    pub fn create_session(&mut self, mtu: i16, addr: SocketAddr) {
        self.sessions.insert(
            addr.to_string(),
            Session::new(
                addr, 
                0,
                self.server_guid,
                mtu
            )
        );
    }

    // pub async fn send

    pub async fn recv_frame_set(
        &mut self,
        _packet_id: u8,
        mut bufin: MsgBuffer,
        client: SocketAddr,
    ) {
        // test bytes: [132, 0, 0, 0, 64, 0, 144, 0, 0, 0, 9, 131, 237, 153, 211, 18, 169, 106, 213, 0, 0, 0, 2, 56, 60, 233, 205, 0]
        let sequence = bufin.read_u24_le_bytes();

        let mut frame_set: FrameSet = FrameSet {
            index: sequence,
            frames: vec![],
        };

        loop {
            if bufin.at_end() {
                break;
            }

            frame_set.frames.push(Frame::parse(&mut bufin))
        }

        let sess = self
            .sessions
            .get_mut(&client.to_string())
            .unwrap();

        let packets_to_send = sess.recv_frame_set(frame_set).await;

        for mut packet in packets_to_send {
            self.socket.send_to(packet.get_bytes(), client).await;
            let bytes = packet.get_bytes();
            trace!("SENT = {bytes:?}");
        }
    }

    pub fn get_unix_milis(&self) -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Uhm... excuse me");
        since_the_epoch.as_millis()
    }

    pub async fn read_message(&mut self) -> Option<(Packet, SocketAddr)> {
        let (size, client) = match self.socket.recv_from(&mut self.buf).await {
            Ok((packetsize, client)) => (packetsize, client),
            Err(_e) => return None, // panic!("recv function failed: {e:?}"),
        };

        let packet_id = self.buf[0];
        let mut body = MsgBuffer::from(self.buf[1..size].to_vec());

        match packet_id {
            0x01 | 0x02 => {
                let offping = OfflinePing::from_buffer(&mut body);

                let offpong = OfflinePong {
                    timestamp: offping.timestamp,
                    server_guid: self.server_guid,
                    magic: offping.magic,
                    server_name: self.get_server_name(),
                };
        
                self.socket.send_packet(0x1c, &mut offpong.to_buffer(), client).await;
                return None;
            },
            0x05 => {
                trace!("0x{packet_id} RECV = {:?}", body.get_bytes());

                let request1 = OfflineConnReq1::from_buffer(&mut body);

                self.create_session(request1.mtu, client);

                let reply1 = OfflineConnRep1 {
                    magic: request1.magic,
                    server_guid: self.server_guid,
                    use_security: false,
                    mtu: request1.mtu,
                };
        
                self.socket.send_packet(0x06, &mut reply1.to_buffer(), client).await;
                return None;
            }
            _ => {}
        }

        trace!("0x{packet_id} RECV = {:?}", body.get_bytes());  // rename to body

        Some((Packet {packet_id, timestamp: self.get_unix_milis(), body}, client))
    }

    pub async fn mainloop(&mut self) {
        loop {
            let last_update_time = self.get_unix_milis();

            while self.get_unix_milis() - last_update_time < 50 {
                let (packet, client) = match self.read_message().await {
                    Some((packet, client)) => (packet, client),
                    None => continue,
                };

                let sess = self.sessions.get_mut(&client.to_string()).unwrap();
                sess.recv(packet);
            }

            for (_, sess) in self.sessions.iter_mut() {
                sess.update().await;
            }

            for (_, sess) in self.sessions.iter_mut() {
                let mut packets = std::mem::replace(&mut sess.send_queue, vec![]);
                for packet in packets.iter_mut() {
                    self.socket.send_packet(packet.packet_id, &mut packet.body, sess.sockaddr).await;
                }
            }
        }
    }
}
