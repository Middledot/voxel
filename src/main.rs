mod config;
mod raknet;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

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

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({l})} {d(%d-%m-%Y %H:%M:%S)} [{f}:{L:<3}] {m}\n",
        )))
        .build();
    let logconfig = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
        .unwrap();
    let _handle = log4rs::init_config(logconfig).unwrap();

    let mut raknet_server = raknet::server::RakNetListener::new(config).await; // later, make config reference for raknet, and VoxelServer owner
    raknet_server.mainloop().await;
}
