#![warn(unreachable_code)]

use rust_raknet::RaknetListener;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

pub struct VoxelServer {
    listener: RaknetListener,
    port: u16,
    motd: String,
    version: String,
    protocol_version: String,
}
// pub enum Gamemode {
//     Survival = 0
// }


impl VoxelServer {
    pub async fn init(port: u16, motd: String) -> Self {
        let listener = RaknetListener::bind(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
            // Look more into how I can put a string here
        ).await.unwrap();
        let me = Self {
            listener: listener,
            port: port,
            motd: motd,
            version: "1.20.31".to_string(),
            protocol_version: "618".to_string()
        };
        return me
    }

    pub async fn close(&mut self) {
        self.listener.close().await.unwrap();
    }

    pub async fn set(&mut self) {
        self.listener.set_motd(
            self.motd.as_ref(),
            20,
            self.protocol_version.as_ref(),
            self.version.as_ref(),
            "Survival",
            self.port,
        ).await;
    }

    pub async fn mainloop(&mut self) {
        self.set().await;
        self.listener.listen().await;
        loop {
            let socket = self.listener.accept().await.unwrap();
            let buf = socket.recv().await.unwrap(); // .into_iter().;
            println!("{:?}", buf);
            if buf[0] == 0xfe {
                println!("{:x?}", buf);
                // socket.send();
            }
        }
        self.close().await;
    }
}
