use omnistreams::{Producer, RangeProducerBuilder, MapConduit, ProducerEvent};


fn main() {

    omnistreams::runtime::run(|| {

        let producer = RangeProducerBuilder::new()
            .start(5)
            .stop(10)
            .build();
        let conduit = MapConduit::new(|x| x*x);

        let mut square_producer = producer
            .pipe_through(conduit);

        square_producer.request(1);

        square_producer.events().for_each(move |event| {
            match event {
                ProducerEvent::Data(value) => {
                    println!("{}", value);
                    square_producer.request(1);
                },
                _ => {
                }
            }
        });
    });
}
