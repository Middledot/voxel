/// raknet/server.rs
/// ================
///
/// The server, one who handles RakNet packets.
///
/// Reference: https://wiki.vg/Raknet_Protocol
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::net::UdpSocket;

use log::trace;

use super::objects::{Frame, MsgBuffer, datatypes::to_address_bytes};
use super::packets::*;
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
    where
        T: Serialize,
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

        println!("{:?}", offline_ping.client_guid);
        println!("{:?}", self.server_guid);

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

        let new_packet_id = frame_set[0].body[0];
        let new_bufin = MsgBuffer::from(frame_set[0].body[1..].to_vec());

        let output = self.get_event_func(new_packet_id)(&self, new_packet_id, new_bufin, client);
    }

    pub fn conn_req(&self, packet_id: u8, mut bufin: MsgBuffer, client: SocketAddr) -> Vec<u8> {
        let apparently_some_unknown_guid = bufin.read_i64_be_bytes();
        let timestamp = bufin.read_i64_be_bytes();

        println!("{:?}", apparently_some_unknown_guid);

        let mut bufout = MsgBuffer::new();
        bufout.write_address(&client);
        bufout.write_i16_be_bytes(&0);  // like, ok
        let mystery_address = to_address_bytes(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 19132)
        );
        for _ in 0..10 {
            bufout.write(&mystery_address);
        }
        bufout.write_i64_be_bytes(&timestamp);
        bufout.write_i64_be_bytes(&(SystemTime::now().duration_since(UNIX_EPOCH).expect("Oops").as_millis() as i64));

        *bufout.into_bytes()
        // self.socket.send_to(bufout.into_bytes(), client).expect("Zamn");
        // trace!("SENT = {:?}", bufout.into_bytes());
    }

    pub fn get_event_func(&self, packet_id: u8) -> fn(&Self, u8, MsgBuffer, SocketAddr) {
        let eventfunc = match packet_id {
            0x01 | 0x02 => RakNetServer::unconnected_ping,
            0x05 => RakNetServer::offline_connection_request_1,
            0x07 => RakNetServer::offline_connection_request_2,
            0x09 => RakNetServer::conn_req,
            0x80..=0x8d => RakNetServer::frame_set,
            _ => panic!("There's nothing we can do | Nous pouvons rien faire"),
        };

        eventfunc
    }

    pub fn run_event(&self, packet_id: u8, bufin: MsgBuffer, client: SocketAddr) {
        self.get_event_func(packet_id)(&self, packet_id, bufin, client);
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
