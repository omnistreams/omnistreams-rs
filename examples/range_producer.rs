use omnistreams::{Producer, RangeProducerBuilder, ProducerEvent};


fn main() {

    omnistreams::runtime::run(|| {

        let mut producer = RangeProducerBuilder::new()
            .start(5)
            .stop(10)
            .build();

        producer.request(1);

        producer.events().for_each(move |event| {
            match event {
                ProducerEvent::Data(value) => {
                    println!("{}", value);
                    producer.request(1);
                },
                _ => {
                }
            }
        });
    });
}
