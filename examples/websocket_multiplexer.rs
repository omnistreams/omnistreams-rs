use omnistreams::{EventEmitter, Acceptor, WebSocketAcceptorBuilder, Multiplexer};
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
            let mut mux = Multiplexer::new(transport);

            let events = mux.events().unwrap();

            tokio::spawn(events.for_each(|receiver| {
                println!("I don't believe it");
                Ok(())
            })
            .map_err(|e| {
                eprintln!("{:?}", e);
            }));

            Ok(())
        })
        .map_err(|e| {
            eprintln!("{:?}", e);
        }));

        Ok(())
    }));
}
