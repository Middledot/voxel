use rand::Rng;
use std::net::UdpSocket;
use std::net::SocketAddr;
// use std::io::{Read, Result};

use crate::config::Config;
use crate::raknet::datatypes;

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

    pub fn unconnected_ping(&self, body: Vec<u8>, client: SocketAddr) {
        // I don't like how this looks but it's Rust so it must be fine
        // TODO: I have a feeling this code is somehow inefficient (using vectors a lot and whatnot)
        // probably look at again at some point
        let (body, client_timestamp) = datatypes::read_i64_be_bytes(body);
        let (body, magic) = datatypes::read_magic(body);
        let (_body, client_guid) = datatypes::read_i64_be_bytes(body);

        let mut buffer: Vec<u8> = vec![];
        buffer.push(0x1c);
        buffer = datatypes::write_i64_be_bytes(&client_timestamp, buffer);
        buffer = datatypes::write_i64_be_bytes(&self.server_guid, buffer);
        buffer = datatypes::write_magic(&magic, buffer);

        // self.server_name.unwrap_or("".to_string());
        let server_name = self.get_server_name();

        let server_name: Vec<u8> = server_name.as_bytes().to_vec();
        let server_name_len = (server_name.len()) as i16;
        buffer = datatypes::write_i16_be_bytes(&server_name_len, buffer);
        buffer.extend_from_slice(&server_name);

        self.socket.send_to(&buffer, client).expect("Sending packet failed");
        println!("SENT = {:?}", &buffer);
    }

    pub fn open_conn_req_1(&self, body: Vec<u8>, client: SocketAddr) {
        let (body, magic) = datatypes::read_magic(body);
        let protocol = body[0];  // magic value, unknown use (always 11)
        let mtu = (body[1..].len() + 46) as i16;

        let mut buffer: Vec<u8> = vec![];
        buffer.push(0x06);
        buffer = datatypes::write_magic(&magic, buffer);
        buffer = datatypes::write_i64_be_bytes(&self.server_guid, buffer);
        buffer.push(0x00);  // boolean (false)
        buffer = datatypes::write_i16_be_bytes(&mtu, buffer);

        self.socket.send_to(&buffer, client).expect("Sending packet failed");
        println!("SENT = {:?}", &buffer);
    }

    pub fn open_conn_req_2(&self, body: Vec<u8>, client: SocketAddr) {
        let (body, magic) = datatypes::read_magic(body);
        let (body, server_address) = datatypes::read_address(body);
        let (body, mtu) = datatypes::read_i16_be_bytes(body);
        let (body, client_guid) = datatypes::read_i64_be_bytes(body);

        let mut buffer: Vec<u8> = vec![];
        buffer.push(0x08);
        buffer = datatypes::write_magic(&magic, buffer);
        buffer = datatypes::write_i64_be_bytes(&self.server_guid, buffer);
        // TODO: write address
        buffer = datatypes::write_i16_be_bytes(&mtu, buffer);
        buffer.push(0);  // disable encryption // TODO: look into?

        self.socket.send_to(&buffer, client). expect("Sending packet failed");
        println!("SENT = {:?}", &buffer);
    }

    pub fn mainloop(&self) {
        let mut buf: [u8; 1024] = [0; 1024];  // 1kb

        loop {
            let (packetsize, client) = match self.socket.recv_from(&mut buf) { //.expect("Zamn");
                Ok((packetsize, client)) => (packetsize, client),
                Err(_e) => continue  // panic!("recv function failed: {e:?}"),
            };
            println!("RECV = {:?}", &buf[..packetsize]);

            match buf[0] {
                0x01 | 0x02 => self.unconnected_ping(buf[1..].to_vec(), client),
                0x05 => self.open_conn_req_1(buf[1..].to_vec(), client),
                0x07 => self.open_conn_req_2(buf[1..].to_vec(), client),
                _ => panic!("There's nothing we can do | Nous pouvons rien faire")
            }
        }
    }
}
