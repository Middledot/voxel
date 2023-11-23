use super::raknet::server::RakNetListener;
use super::config::Config;
use std::thread;

pub struct VoxelServer {
    // listener: RakNetListener,
    version: String,
    protocol_version: String,
}


impl VoxelServer {
    pub async fn init() -> Self {
        // let listener = RakNetListener::new(
        //     config
        // ).await;
        let me = Self {
            version: "1.20.31".to_string(),  // dunno what this is yet
            protocol_version: "622".to_string()
        };
        return me
    }

    // pub async fn close(&mut self) {
    //     self.listener.close().await.unwrap();
    // }

    pub async fn run(&mut self, config: Config) {
        let raknet_thread = thread::spawn(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut listener = RakNetListener::new(config).await;
                listener.mainloop().await;
            })
        });

        loop {}

        let _ = raknet_thread.join();
        // self.close().await;
    }
}
