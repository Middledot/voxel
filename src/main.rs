mod raknet;
mod config;

#[tokio::main]
async fn main() {
    // let mut listener = RaknetListener::bind(
    //     "127.0.0.1:25565"
    //     .to_socket_addrs()
    //     .collect::<Vec<&SocketAddr>>()
    //     .into_iter()
    //     .nth(0)
    // ).await.unwrap();

    // let mut server: VoxelServer = VoxelServer::init(
    //     19132,
    //     "Hi :)".to_string()
    // ).await;

    // server.mainloop().await;
    // -------------------------------------------------------------------------
    // let socket = std::net::UdpSocket::bind("127.0.0.1:19132").expect("Zamn");
    // let mut buf: [u8; 1024] = [0; 1024];

    // loop {
    //     let (packetsize, client) = match socket.recv_from(&mut buf) { //.expect("Zamn");
    //         Ok((packetsize, client)) => (packetsize, client),
    //         Err(e) => panic!("recv function failed: {e:?}"),
    //     };
    //     println!("{:?}", &buf[..packetsize]);

    //     match buf[0] {
    //         // 0x01 | 0x02 => 
    //         _ => panic!("idk")
    //     }
    // }
    let config = config::Config::parse();

    let raknet_server = raknet::server::RakNetServer::bind(config);  // later, make config reference for raknet, and VoxelServer owner
    raknet_server.mainloop();
}
