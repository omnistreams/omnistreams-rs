use omnistreams::{Producer, RangeProducerBuilder, MapConduit, ProducerEvent};
use futures::future::lazy;
use tokio::prelude::*;


fn main() {

    tokio::run(lazy(|| {

        let producer = RangeProducerBuilder::new()
            .start(5)
            .stop(10)
            .build();
        let conduit = MapConduit::new(|x: i64| x*x);

        let mut square_producer = producer
            .pipe_conduit(conduit);
            //.pipe_conduit(MapConduit::new(|x: i64| {
            //    format!("val: {:?}", x)
            //}));

        let events = square_producer.event_stream().unwrap();

        tokio::spawn(events.for_each(|event| {
            match event {
                ProducerEvent::Data(value) => {
                    println!("{}", value);
                },
                _ => {
                }
            }
            Ok(())
        })
        .map_err(|_| {}));

        square_producer.request(10);
            
        Ok(())
    }));
}
