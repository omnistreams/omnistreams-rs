use omnistreams::{Consumer, WriteAdapter};
use tokio::io;
use tokio::prelude::*;


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

struct Pipe<T> 
    where T: Consumer + Future<Item=(), Error=io::Error> + Send + 'static,
{
    consumer_future: T,
}

impl<T> Pipe<T>
    where T: Consumer + Future<Item=(), Error=io::Error> + Send + 'static,
{
    fn new(producer: impl Producer, consumer_future: T) -> Pipe<T> {

        let consumer_message_tx = consumer_future.message_tx();

        Pipe {
            consumer_future,
        }
    }
}

impl<T> Future for Pipe<T>
    where T: Consumer + Future<Item=(), Error=io::Error> + Send + 'static,
{
    type Item = ();
    type Error = i32;

    fn poll(&mut self) -> Poll<(), i32> {
        Ok(Async::Ready(()))
        //Ok(Async::NotReady)
    }
}


fn main() {

    let file_writer = tokio::fs::File::create("foo.txt");
    let consumer = WriteAdapter::new(file_writer);
    println!("{:?}", consumer);
    //let consumer = consumer.map_err(|_| {});

    let producer = DummyProducer::new();

    let pipe = Pipe::new(producer, consumer).map_err(|e| {
        eprintln!("{:?}", e);
    });

    tokio::run(pipe);
}
