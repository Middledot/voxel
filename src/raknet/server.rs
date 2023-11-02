use rand::Rng;
use std::net::SocketAddr;
use std::net::UdpSocket;

use crate::config::Config;
use super::objects::{Frame, MsgBuffer};

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

    pub fn unconnected_ping(&self, packet_id: u8, mut bufin: MsgBuffer, client: SocketAddr) {
        let client_timestamp = bufin.read_i64_be_bytes();
        let magic = bufin.read_magic();
        let _client_guid = bufin.read_i16_be_bytes();

        let mut bufout = MsgBuffer::new();
        bufout.write_byte(0x1c);
        bufout.write_i64_be_bytes(&client_timestamp);
        bufout.write_i64_be_bytes(&self.server_guid);
        bufout.write_magic(&magic);

        let server_name = self.get_server_name();
        let server_name: Vec<u8> = server_name.as_bytes().to_vec();
        let server_name_len = (server_name.len()) as i16;

        bufout.write_i16_be_bytes(&server_name_len);
        bufout.write(&server_name);

        self.socket
            .send_to(bufout.into_bytes(), client)
            .expect("Sending packet failed");
        println!("SENT = {:?}", bufout.into_bytes());
    }

    pub fn offline_connection_request_1(
        &self,
        packet_id: u8,
        mut bufin: MsgBuffer,
        client: SocketAddr,
    ) {
        let magic = bufin.read_magic();
        let _protocol = bufin.read_byte(); // mysterious magical mystical value, unknown use (always 11)
        let mtu = (bufin.len_rest() + 46) as i16;

        let mut bufout = MsgBuffer::new();
        bufout.write_byte(0x06);
        bufout.write_magic(&magic);
        bufout.write_i64_be_bytes(&self.server_guid);
        bufout.write_byte(0x00); // boolean (false)
        bufout.write_i16_be_bytes(&mtu);

        self.socket
            .send_to(bufout.into_bytes(), client)
            .expect("Sending packet failed");
        println!("SENT = {:?}", bufout.into_bytes());
    }

    pub fn offline_connection_request_2(
        &self,
        packet_id: u8,
        mut bufin: MsgBuffer,
        client: SocketAddr,
    ) {
        let magic = bufin.read_magic();
        let _server_address = bufin.read_address();
        let mtu = bufin.read_i16_be_bytes();
        let _client_guid = bufin.read_i64_be_bytes();

        let mut bufout = MsgBuffer::new();
        bufout.write_byte(0x08);
        bufout.write_magic(&magic);
        bufout.write_i64_be_bytes(&self.server_guid);
        bufout.write_address(&client);
        bufout.write_i16_be_bytes(&mtu);
        bufout.write_byte(0); // disable encryption // TODO: look into? what is this?

        self.socket
            .send_to(bufout.into_bytes(), client)
            .expect("Sending packet failed");
        println!("SENT = {:?}", bufout.into_bytes());
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
                //.expect("Zamn");
                Ok((packetsize, client)) => (packetsize, client),
                Err(_e) => continue, // panic!("recv function failed: {e:?}"),
            };
            println!("RECV = {:?}", &buf[..packetsize]);
            let bufin = MsgBuffer::from(buf[1..packetsize].to_vec());

            self.run_event(buf[0], bufin, client);
        }
    }
}
