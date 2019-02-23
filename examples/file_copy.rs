use omnistreams::{Producer, ReadAdapter, WriteAdapter};
use futures::future::lazy;


fn main() {

    tokio::run(lazy(|| {

        let file_reader = tokio::fs::File::open("in.txt");
        let producer = ReadAdapter::new(file_reader);

        let file_writer = tokio::fs::File::create("out.txt");
        let consumer = WriteAdapter::new(file_writer);

        producer.pipe(consumer);

        Ok(())
    }));
}
