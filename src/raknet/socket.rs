use super::objects::MsgBuffer;
use std::io::Error;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use log::trace;

pub struct Socket {
    pub udpsock: UdpSocket,
}

impl Socket {
    pub async fn bind(addr: String) -> Self {
        let udpsock = UdpSocket::bind(addr).await.unwrap();
        Self { udpsock }
    }

    pub async fn send_packet(&self, packet_id: u8, packet: &mut MsgBuffer, client: SocketAddr) {
        let serialized = packet;
        let body = serialized.get_bytes();
        let mut bytes = vec![packet_id];
        bytes.extend_from_slice(body);

        self.send_to(&bytes, client).await;

        if packet_id != 0x1c {
            trace!("0x{packet_id} SENT = {:?}", &bytes);
        }
    }

    pub async fn send_to(&self, buf: &[u8], target: SocketAddr) {
        self.udpsock
            .send_to(buf, target)
            .await
            .unwrap_or_else(|_| panic!("Failed to send packet to {}", target));
    }

    pub fn try_recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        self.udpsock.try_recv_from(buf)
    }
}
