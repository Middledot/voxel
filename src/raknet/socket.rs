use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::io::Error;
use super::objects::MsgBuffer;

use log::trace;

pub struct Socket {
    pub udpsock: UdpSocket,
}

impl Socket {
    pub async fn bind(addr: String) -> Self {
        let udpsock = UdpSocket::bind(addr).await.unwrap();
        Self {
            udpsock
        }
    }

    pub async fn send_packet(&self, packet_id: u8, packet: &mut MsgBuffer, client: SocketAddr) {
        let serialized = packet;
        let body = serialized.get_bytes();
        let mut bytes = vec![packet_id];
        bytes.extend_from_slice(body);

        self.send_to(&bytes, client).await;

        println!("ok {}", packet_id);
        if !(packet_id == 0x1c) {
            trace!("0x{packet_id} SENT = {body:?}");
        }
    }

    pub async fn send_to(&self, buf: &[u8], target: SocketAddr) {
        self.udpsock.send_to(buf, target).await.expect(format!("Failed to send packet to {}", target.to_string()).as_str());
    }

    // pub async fn poll_recv_from(&self, buf: &mut [u8]) {
    //     self.udpsock.poll_recv_from()
    // }

    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        self.udpsock.recv_from(buf).await
    }
}
