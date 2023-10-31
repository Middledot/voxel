use rand::Rng;
use std::net::UdpSocket;
use std::net::SocketAddr;
// use std::io::{Read, Result};

use crate::config::Config;
use crate::raknet::datatypes::MsgBuffer;

pub struct RakNetServer {
    socket: UdpSocket,
    server_guid: i64,
    config: Config
}

impl RakNetServer {
    pub fn bind(config: Config) -> Self {
        let mut rng = rand::thread_rng();
        let random_number: i64 = rng.gen_range(1..=i64::MAX);

        Self {
            socket: std::net::UdpSocket::bind("127.0.0.1:".to_string() + config.get_property("server-port")).expect("Zamn"),
            server_guid: random_number,
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
            self.config.get_property("server-portv6").as_str()
        ].join(";")
    }

    pub fn unconnected_ping(&self, mut body: MsgBuffer, client: SocketAddr) {
        let client_timestamp = body.read_i64_be_bytes();
        let magic = body.read_magic();
        let _client_guid = body.read_i16_be_bytes();

        let mut buffer = MsgBuffer::new();
        buffer.write_byte(0x1c);
        buffer.write_i64_be_bytes(&client_timestamp);
        buffer.write_i64_be_bytes(&self.server_guid);
        buffer.write_magic(&magic);

        let server_name = self.get_server_name();
        let server_name: Vec<u8> = server_name.as_bytes().to_vec();
        let server_name_len = (server_name.len()) as i16;

        buffer.write_i16_be_bytes(&server_name_len);
        buffer.write(&server_name);

        self.socket.send_to(buffer.into_bytes(), client).expect("Sending packet failed");
        println!("SENT = {:?}", buffer.into_bytes());
    }

    pub fn offline_connection_request_1(&self, mut body: MsgBuffer, client: SocketAddr) {
        let magic = body.read_magic();
        let _protocol = body.read_byte();  // mysterious magical mystical value, unknown use (always 11)
        let mtu = (body.rest_len() + 46) as i16;

        let mut buffer = MsgBuffer::new();
        buffer.write_byte(0x06);
        buffer.write_magic(&magic);
        buffer.write_i64_be_bytes(&self.server_guid);
        buffer.write_byte(0x00);  // boolean (false)
        buffer.write_i16_be_bytes(&mtu);

        self.socket.send_to(buffer.into_bytes(), client).expect("Sending packet failed");
        println!("SENT = {:?}", buffer.into_bytes());
    }

    pub fn offline_connection_request_2(&self, mut body: MsgBuffer, client: SocketAddr) {
        let magic = body.read_magic();
        let _server_address = body.read_address();
        let mtu = body.read_i16_be_bytes();
        let _client_guid = body.read_i64_be_bytes();

        let mut buffer = MsgBuffer::new();
        buffer.write_byte(0x08);
        buffer.write_magic(&magic);
        buffer.write_i64_be_bytes(&self.server_guid);
        buffer.write_address(&client);
        buffer.write_i16_be_bytes(&mtu);
        buffer.write_byte(0);  // disable encryption // TODO: look into?

        self.socket.send_to(buffer.into_bytes(), client). expect("Sending packet failed");
        println!("SENT = {:?}", buffer.into_bytes());
    }

    pub fn frame_set(&self) {
        // nothing yet
    }

    pub fn mainloop(&self) {
        let mut buf = [0u8; 1024];  // 1kb

        loop {
            let (packetsize, client) = match self.socket.recv_from(&mut buf) { //.expect("Zamn");
                Ok((packetsize, client)) => (packetsize, client),
                Err(_e) => continue  // panic!("recv function failed: {e:?}"),
            };
            println!("RECV = {:?}", &buf[..packetsize]);
            let body = MsgBuffer::from(buf[1..packetsize].to_vec());

            match buf[0] {
                0x01 | 0x02 => self.unconnected_ping(body, client),
                0x05 => self.offline_connection_request_1(body, client),
                0x07 => self.offline_connection_request_2(body, client),
                0x80..=0x8d => self.frame_set(),
                _ => panic!("There's nothing we can do | Nous pouvons rien faire")
            }
        }
    }
}
