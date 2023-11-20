/// raknet/server.rs
/// ================
///
/// The server, one who handles RakNet packets.
///
/// Reference: https://wiki.vg/Raknet_Protocol
use rand::Rng;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread;

use log::trace;

use super::objects::datatypes::get_unix_milis;
use super::objects::msgbuffer::Packet;
use super::objects::MsgBuffer;
use super::packets::*;
use super::session::Session;
use super::socket::Socket;
use crate::config::Config;

pub struct RakNetServer {
    socket: Socket,
    server_guid: i64,
    config: Config,
    sessions: HashMap<String, Session>,
    buf: [u8; 2048],
}

impl RakNetServer {
    pub async fn new(config: Config) -> Self {
        let socket =
            Socket::bind("127.0.0.1:".to_string() + config.get_property("server-port")).await;

        Self {
            socket,
            server_guid: rand::thread_rng().gen_range(1..=i64::MAX),
            config,
            sessions: HashMap::new(),
            buf: [0u8; 2048],
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
        let sess = Session::new(addr, 0, self.server_guid, mtu);

        self.sessions.insert(addr.to_string(), sess);
    }

    pub async fn read_message(&mut self) -> Option<(Packet, SocketAddr)> {
        let (size, client) = match self.socket.try_recv_from(&mut self.buf) {
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

                self.socket
                    .send_packet(0x1c, &mut offpong.to_buffer(), client)
                    .await;
                return None;
            }
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

                self.socket
                    .send_packet(0x06, &mut reply1.to_buffer(), client)
                    .await;
                return None;
            }
            0x07 => {
                trace!("0x{packet_id} RECV = {:?}", body.get_bytes());

                let sess = self.sessions.get_mut(&client.to_string()).unwrap();

                let request2 = OfflineConnReq2::from_buffer(&mut body);

                let reply2 = OfflineConnRep2 {
                    magic: request2.magic,
                    server_guid: sess.server_guid,
                    client_address: sess.sockaddr,
                    mtu: sess.mtu as i16,
                    use_encryption: false, // disable encryption // TODO: look into? what is this?
                };

                sess.guid = request2.client_guid;

                self.socket
                    .send_packet(0x08, &mut reply2.to_buffer(), client)
                    .await;
                return None;
            }
            _ => {}
        }

        trace!("0x{packet_id} RECV = {:?}", body.get_bytes()); // rename to body

        Some((
            Packet {
                packet_id,
                timestamp: get_unix_milis(),
                body,
            },
            client,
        ))
    }

    pub async fn mainloop(&mut self) {
        loop {
            let last_update_time = get_unix_milis();

            while get_unix_milis() - last_update_time < 50 {
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
                    self.socket
                        .send_packet(packet.packet_id, &mut packet.body, sess.sockaddr)
                        .await;
                }
            }
        }
    }
}
