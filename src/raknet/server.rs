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

    pub fn unconnected_ping(&self, mut bufin: MsgBuffer, client: SocketAddr) {
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

        self.socket.send_to(bufout.into_bytes(), client).expect("Sending packet failed");
        println!("SENT = {:?}", bufout.into_bytes());
    }

    pub fn offline_connection_request_1(&self, mut bufin: MsgBuffer, client: SocketAddr) {
        let magic = bufin.read_magic();
        let _protocol = bufin.read_byte();  // mysterious magical mystical value, unknown use (always 11)
        let mtu = (bufin.len_rest() + 46) as i16;

        let mut bufout = MsgBuffer::new();
        bufout.write_byte(0x06);
        bufout.write_magic(&magic);
        bufout.write_i64_be_bytes(&self.server_guid);
        bufout.write_byte(0x00);  // boolean (false)
        bufout.write_i16_be_bytes(&mtu);

        self.socket.send_to(bufout.into_bytes(), client).expect("Sending packet failed");
        println!("SENT = {:?}", bufout.into_bytes());
    }

    pub fn offline_connection_request_2(&self, mut bufin: MsgBuffer, client: SocketAddr) {
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
        bufout.write_byte(0);  // disable encryption // TODO: look into?

        self.socket.send_to(bufout.into_bytes(), client). expect("Sending packet failed");
        println!("SENT = {:?}", bufout.into_bytes());
    }

    pub fn frame_set(&self, mut bufin: MsgBuffer, client: SocketAddr) {
        let sequence = bufin.read_u24_le_bytes();

        let flags = bufin.read_byte();
        let bitlength = bufin.read_u16_be_bytes();
        // TODO: check if these parameters appear as 0s or don't appear
        let rel_frameindex = bufin.read_u24_le_bytes();
        let seq_frameindex = bufin.read_u24_le_bytes();

        let ord_frameindex = bufin.read_u24_le_bytes();
        let ord_chnl = bufin.read_byte();

        // let compound_size = 
        let compound_id = bufin.read_i16_be_bytes();
        // let index = 

        let body = bufin.read_rest();
    }

    pub fn run_event(&self, id: u8, bufin: MsgBuffer, client: SocketAddr) {
        match id {
            0x01 | 0x02 => self.unconnected_ping(bufin, client),
            0x05 => self.offline_connection_request_1(bufin, client),
            0x07 => self.offline_connection_request_2(bufin, client),
            0x80..=0x8d => self.frame_set(bufin, client),
            _ => panic!("There's nothing we can do | Nous pouvons rien faire")
        }
    }

    pub fn mainloop(&self) {
        let mut buf = [0u8; 1024];  // 1kb

        loop {
            let (packetsize, client) = match self.socket.recv_from(&mut buf) { //.expect("Zamn");
                Ok((packetsize, client)) => (packetsize, client),
                Err(_e) => continue  // panic!("recv function failed: {e:?}"),
            };
            println!("RECV = {:?}", &buf[..packetsize]);
            let bufin = MsgBuffer::from(buf[1..packetsize].to_vec());

            self.run_event(buf[0], bufin, client);
        }
    }
}
