use omnistreams::{
    Producer, RangeProducerBuilder, MapConduit, WriteAdapter,
};
use futures::future::lazy;


fn main() {

    tokio::run(lazy(|| {

        let file_writer = tokio::fs::File::create("squares.txt");

        let mut producer = RangeProducerBuilder::new()
            .start(0)
            .stop(100)
            .build();

        producer.request(200);

        producer
            .pipe_conduit(MapConduit::new(|x| x*x))
            .pipe_conduit(MapConduit::new(|x| format!("{:?}\n", x).as_bytes().to_vec()))
            //.pipe(WriteAdapter::new(futures::future::ok(tokio::io::stdout())));
            .pipe(WriteAdapter::new(file_writer));
            
        Ok(())
    }));
}
