use omnistreams::{Acceptor, WebSocketAcceptorBuilder, Multiplexer};
use futures::future::lazy;
use tokio::prelude::*;


fn main() {
    tokio::run(lazy(|| {
        let mut acceptor = WebSocketAcceptorBuilder::new()
            .port(9001)
            .build();

        let transports = acceptor.transports().expect("no transports");

        tokio::spawn(transports.for_each(|transport| {
            println!("{:?}", transport);
            let mux = Multiplexer::new(transport);
            Ok(())
        })
        .map_err(|e| {
            eprintln!("{:?}", e);
        }));

        Ok(())
    }));
}
