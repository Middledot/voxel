/// raknet/server.rs
/// ================
///
/// The server, one who handles RakNet packets.
///
/// Reference: https://wiki.vg/Raknet_Protocol
use rand::Rng;
use std::net::{SocketAddr, IpAddr};
use tokio::net::UdpSocket;
use std::collections::HashMap;

use log::trace;

use super::objects::{Frame, MsgBuffer};
use super::packets::*;
use super::session::{Session, FrameSet};
use crate::config::Config;

pub struct RakNetServer {
    socket: UdpSocket,
    server_guid: i64,
    config: Config,
    sessions: HashMap<(IpAddr, u16), Session>
}

impl RakNetServer {
    pub async fn bind(config: Config) -> Self {
        let socket = UdpSocket::bind(
            "127.0.0.1:".to_string() + config.get_property("server-port"),
        )
        .await
        .expect("Failed to bind to port");

        Self {
            socket: socket,
            server_guid: rand::thread_rng().gen_range(1..=i64::MAX),
            config: config,
            sessions: HashMap::new(),
        }
    }

    pub fn get_server_name(&mut self) -> String {
        let motd = self.config.get_property("server-name");

        // so picky I don't get it smh
        return vec![
            "MCPE",
            &motd,
            "622",
            "1.20.40",
            self.sessions.len().to_string().as_str(),
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

    pub async fn send_packet<T>(&mut self, packet_id: u8, packet: &T, client: SocketAddr)
    where
        T: Serialize,
    {
        let mut serialized = packet.serialize();
        let body = serialized.into_bytes();
        let mut bytes = vec![packet_id];
        bytes.extend_from_slice(&body);

        self.socket
            .send_to(&bytes, client)
            .await
            .expect("Sending packet failed");

        if !packet_id == 0x1c {
            trace!("0x{packet_id} SENT = {body:?}");
        }
    }

    pub async fn unconnected_ping(&mut self, packet_id: u8, mut bufin: MsgBuffer, client: SocketAddr) {
        let offline_ping = OfflinePing::deserialise(&mut bufin);

        let offline_pong = OfflinePong {
            timestamp: offline_ping.timestamp,
            server_guid: self.server_guid,
            magic: offline_ping.magic,
            server_name: self.get_server_name(),
        };

        self.send_packet(0x1c, &offline_pong, client).await;
    }

    pub async fn offline_connection_request_1(
        &mut self,
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

        self.send_packet(0x06, &offline_conn_rep_1, client).await;
    }

    pub async fn offline_connection_request_2(
        &mut self,
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

        self.send_packet(0x08, &offline_conn_rep_2, client).await;

        let user = Session::new(client, offline_conn_req_2.client_guid, offline_conn_req_2.mtu);

        self.sessions.insert(
            (user.sockaddr.ip(), user.sockaddr.port()),
            user,
        );
    }

    pub async fn recv_frame_set(&mut self, packet_id: u8, mut bufin: MsgBuffer, client: SocketAddr) {
        // test bytes: [132, 0, 0, 0, 64, 0, 144, 0, 0, 0, 9, 131, 237, 153, 211, 18, 169, 106, 213, 0, 0, 0, 2, 56, 60, 233, 205, 0]
        let sequence = bufin.read_u24_le_bytes();

        let mut frame_set: FrameSet = FrameSet {index: sequence, frames: vec![]};

        loop {
            if bufin.at_end() {
                break;
            }

            frame_set.frames.push(Frame::parse(&mut bufin))
        }

        let sess = self.sessions.get_mut(&(client.ip(), client.port())).unwrap();

        let packets_to_send = sess.recv_frame_set(frame_set).await;

        for mut packet in packets_to_send {
            self.socket.send_to(packet.into_bytes(), client).await.expect("damn");
            let bytes = packet.into_bytes();
            trace!("SENT = {bytes:?}");
        }
    }

    pub async fn call_offline_event(&mut self, packet_id: u8, bufin: MsgBuffer, client: SocketAddr) {
        match packet_id {
            0x01 | 0x02 => self.unconnected_ping(packet_id, bufin, client).await,
            0x05 => self.offline_connection_request_1(packet_id, bufin, client).await,
            0x07 => self.offline_connection_request_2(packet_id, bufin, client).await,
            0x80..=0x8d => self.recv_frame_set(packet_id, bufin, client).await,
        };
    }

    pub async fn call_online_event(&mut self, packet_id: u8, bufin: MsgBuffer, client: SocketAddr) {
        let mut sess = self.sessions.get_mut(&(client.ip(), client.port())).unwrap();
        match packet_id {
            0xC0 => sess.recv_ack(ACK::deserialise(&mut bufin))
        }
    }

    pub async fn mainloop(&mut self) {
        let mut buf = [0u8; 2048]; // 2kb

        loop {
            let (packetsize, client) = match self.socket.recv_from(&mut buf).await {
                Ok((packetsize, client)) => (packetsize, client),
                Err(_e) => continue, // panic!("recv function failed: {e:?}"),
            };
            let packet_id = buf[0];
            let body = &buf[1..packetsize];
            if !(buf[0] == 0x01 || buf[0] == 0x02) {
                trace!("0x{packet_id} RECV = {body:?}");
            }
            let bufin = MsgBuffer::from(body.to_vec());

            match packet_id {
                0x01 | 0x02 | 0x05 | 0x07 | 0x80..=0x8d => self.call_offline_event(packet_id, bufin, client).await,

                _ => panic!("There's nothing we can do | Nous pouvons rien faire"),
            }
        }
    }
}
