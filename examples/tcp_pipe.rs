use omnistreams::{ReadAdapter, WriteAdapter, pipe};
use tokio::net::TcpListener;
use tokio::prelude::*;
use std::net::SocketAddr;



fn main() {
    
    let addr = "127.0.0.1:9001".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    tokio::run(listener.incoming()
        .map_err(|e| eprintln!("failed to accept socket; error = {:?}", e))
        .for_each(|socket| {
            println!("New session:");

            let tcp_reader = futures::future::ok(socket);
            let producer = ReadAdapter::new(tcp_reader);

            let stdout_writer = futures::future::ok(tokio::io::stdout());
            let consumer = WriteAdapter::new(stdout_writer);

            pipe(producer, consumer);
            Ok(())
        }));
}
