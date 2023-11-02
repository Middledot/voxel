mod config;
mod raknet;

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

    let config = config::Config::parse();

    let raknet_server = raknet::server::RakNetServer::bind(config); // later, make config reference for raknet, and VoxelServer owner
    raknet_server.mainloop();

    // // packet_id: 132
    // // sequence: 0, 0, 0
    // let test_bytes = [64, 0, 144, 0, 0, 0, 9, 131, 237, 153, 211, 18, 169, 106, 213, 0, 0, 0, 2, 56, 60, 233, 205, 0];
    // let mut buf = raknet::datatypes::MsgBuffer::from(test_bytes.to_vec());
    // let frame = raknet::datatypes::Frame::parse(&mut buf);
}
