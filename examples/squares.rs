use omnistreams::{Producer, RangeProducerBuilder, MapConduit, ProducerEvent};
use futures::future::lazy;
use tokio::prelude::*;


fn main() {

    tokio::run(lazy(|| {

        let producer = RangeProducerBuilder::new()
            .start(5)
            .stop(10)
            .build();
        let conduit = MapConduit::new(|x| x*x);

        let mut square_producer = producer
            .pipe_through(conduit);

        let events = square_producer.event_stream().unwrap();

        square_producer.request(1);

        tokio::spawn(events.for_each(move |event| {
            match event {
                ProducerEvent::Data(value) => {
                    println!("{}", value);
                    square_producer.request(1);
                },
                _ => {
                }
            }
            Ok(())
        })
        .map_err(|_| {}));
            
        Ok(())
    }));
}
