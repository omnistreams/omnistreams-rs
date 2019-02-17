use omnistreams::{WriteAdapter};
use tokio::prelude::*;


trait Consumer {
}

trait Producer {
}

struct DummyProducer {
}

impl DummyProducer {
    fn new() -> DummyProducer {
        DummyProducer {
        }
    }
}

impl Producer for DummyProducer {
}

struct Pipe<T> {
    consumer: Option<T>,
}

impl<T> Pipe<T> {
    fn new(producer: impl Producer, consumer_future: T) -> Pipe<T> {
        Pipe {
            consumer: None,
        }
    }

    fn pipe(consumer: T) {
    }
}

impl<T> Future for Pipe<T> {
    type Item = ();
    type Error = i32;

    fn poll(&mut self) -> Poll<(), i32> {
        Ok(Async::Ready(()))
        //Ok(Async::NotReady)
    }
}


fn main() {

    let file_writer = tokio::fs::File::create("foo.txt");
    let consumer = WriteAdapter::new(file_writer).map_err(|_| {});

    let producer = DummyProducer::new();

    let pipe = Pipe::new(producer, consumer).map_err(|_| {});

    tokio::run(pipe);
}
