/// raknet/server.rs
/// ================
/// 
/// The server, one who handles RakNet packets.
/// 
/// Reference: https://wiki.vg/Raknet_Protocol

use rand::Rng;
use std::net::SocketAddr;
use std::net::UdpSocket;

use log::trace;

use super::packets::*;
use super::objects::{Frame, MsgBuffer};
use crate::config::Config;

pub struct RakNetServer {
    socket: UdpSocket,
    server_guid: i64,
    config: Config,
}

impl RakNetServer {
    pub fn bind(config: Config) -> Self {
        Self {
            socket: std::net::UdpSocket::bind(
                "127.0.0.1:".to_string() + config.get_property("server-port"),
            )
            .expect("Failed to bind to port"),
            server_guid: rand::thread_rng().gen_range(1..=i64::MAX),
            config: config,
        }
    }

    pub fn get_server_name(&self) -> String {
        let motd = self.config.get_property("server-name");

        // so picky I don't get it smh
        return vec![
            "MCPE",
            &motd,
            "622",
            "1.20.40",
            "0",
            self.config.get_property("max-players").as_str(),
            self.server_guid.to_string().as_str(),
            &motd,
            "Creative",
            "1",
            self.config.get_property("server-port").as_str(),
            self.config.get_property("server-portv6").as_str(),
        ]
        .join(";");
    }

    pub fn send_packet<T>(&self, packet: &T, client: SocketAddr)
        where T: Serialize
    {
        let mut serialized = packet.serialize();
        let body = serialized.into_bytes();
        let mut bytes = vec![T::ID];
        bytes.extend_from_slice(&body);

        self.socket
            .send_to(&bytes, client)
            .expect("Sending packet failed");

        trace!("SENT = {:?}", body);
    }

    pub fn unconnected_ping(&self, packet_id: u8, mut bufin: MsgBuffer, client: SocketAddr) {
        let offline_ping = OfflinePing::deserialise(&mut bufin);

        let offline_pong = OfflinePong {
            timestamp: offline_ping.timestamp,
            server_guid: self.server_guid,
            magic: offline_ping.magic,
            server_name: self.get_server_name(),
        };

        self.send_packet(&offline_pong, client);
    }

    pub fn offline_connection_request_1(
        &self,
        packet_id: u8,
        mut bufin: MsgBuffer,
        client: SocketAddr,
    ) {
        let offline_conn_req_1 = OfflineConnReq1::deserialise(&mut bufin);

        let offline_conn_rep_1 = OfflineConnRep1 {
            magic: offline_conn_req_1.magic,
            server_guid: self.server_guid,
            use_security: false,
            mtu: offline_conn_req_1.mtu,
        };

        self.send_packet(&offline_conn_rep_1, client);
    }

    pub fn offline_connection_request_2(
        &self,
        packet_id: u8,
        mut bufin: MsgBuffer,
        client: SocketAddr,
    ) {
        let offline_conn_req_2 = OfflineConnReq2::deserialise(&mut bufin);

        let offline_conn_rep_2 = OfflineConnRep2 {
            magic: offline_conn_req_2.magic,
            server_guid: self.server_guid,
            client_address: client,
            mtu: offline_conn_req_2.mtu,
            use_encryption: false, // disable encryption // TODO: look into? what is this?
        };

        self.send_packet(&offline_conn_rep_2, client);
    }

    pub fn frame_set(&self, packet_id: u8, mut bufin: MsgBuffer, client: SocketAddr) {
        // test bytes: [132, 0, 0, 0, 64, 0, 144, 0, 0, 0, 9, 131, 237, 153, 211, 18, 169, 106, 213, 0, 0, 0, 2, 56, 60, 233, 205, 0]
        let sequence = bufin.read_u24_le_bytes();

        let mut frame_set: Vec<Frame> = vec![];
        println!("seqs: {:?}", &sequence);

        loop {
            if bufin.at_end() {
                break;
            }

            frame_set.push(Frame::parse(&mut bufin))
        }
    }

    pub fn run_event(&self, packet_id: u8, bufin: MsgBuffer, client: SocketAddr) {
        match packet_id {
            0x01 | 0x02 => self.unconnected_ping(packet_id, bufin, client),
            0x05 => self.offline_connection_request_1(packet_id, bufin, client),
            0x07 => self.offline_connection_request_2(packet_id, bufin, client),
            0x80..=0x8d => self.frame_set(packet_id, bufin, client),
            _ => panic!("There's nothing we can do | Nous pouvons rien faire"),
        }
    }

    pub fn mainloop(&self) {

        let mut buf = [0u8; 1024]; // 1kb

        loop {
            let (packetsize, client) = match self.socket.recv_from(&mut buf) {
                Ok((packetsize, client)) => (packetsize, client),
                Err(_e) => continue, // panic!("recv function failed: {e:?}"),
            };
            trace!("RECV = {:?}", &buf[..packetsize]);
            let bufin = MsgBuffer::from(buf[1..packetsize].to_vec());

            self.run_event(buf[0], bufin, client);
        }
    }
}
