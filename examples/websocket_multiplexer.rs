use omnistreams::{
    Producer, Transport, EventEmitter, Acceptor, WebSocketAcceptorBuilder,
    Multiplexer
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

    tokio::spawn(events.for_each(|producer| {
        handle_producer(producer);
        Ok(())
    })
    .map_err(|e| {
        eprintln!("{:?}", e);
    }));
}

fn handle_producer<P: Producer<Vec<u8>> + Send + 'static>(mut producer: P) {

    let events = producer.event_stream().unwrap();

    tokio::spawn(events.for_each(move |event| {
        println!("receiver event: {:?}", event);
        producer.request(2);
        Ok(())
    })
    .map_err(|e| {
        eprintln!("{:?}", e);
    }));
}
