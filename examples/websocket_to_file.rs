use omnistreams::{
    Producer, Transport, EventEmitter, Acceptor, WebSocketAcceptorBuilder,
    Multiplexer, MultiplexerEvent, WriteAdapter 
};
use futures::future::lazy;
use tokio::prelude::*;


fn main() {
    tokio::run(lazy(|| {
        let mut acceptor = WebSocketAcceptorBuilder::new()
            .port(9001)
            .build();

        let transports = acceptor.transports().expect("no transports");

        tokio::spawn(transports.for_each(|transport| {
            handle_transport(transport); 
            Ok(())
        })
        .map_err(|e| {
            eprintln!("{:?}", e);
        }));

        Ok(())
    }));
}

fn handle_transport<T: Transport + Send + 'static>(transport: T) {
    let mut mux = Multiplexer::new(transport);

    let events = mux.events().unwrap();

    tokio::spawn(events.for_each(|event| {
        match event {
            MultiplexerEvent::Conduit(producer) => {
                let file_writer = tokio::fs::File::create("outfile");
                producer
                    .pipe(WriteAdapter::new(file_writer));
            },
            _ => {
            },
        }
        Ok(())
    })
    .map_err(|e| {
        eprintln!("{:?}", e);
    }));
}
