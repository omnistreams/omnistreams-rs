use omnistreams::{Consumer, ConsumerEvent, WriteAdapter};
use tokio::io;
use tokio::prelude::*;
use futures::future::lazy;


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

//struct Pipe<T> 
//    where T: Consumer + Future<Item=(), Error=io::Error> + Send + 'static,
//{
//    consumer_future: T,
//}
//
//impl<T> Pipe<T>
//    where T: Consumer + Future<Item=(), Error=io::Error> + Send + 'static,
//{
//    fn new(producer: impl Producer, consumer_future: T) -> Pipe<T> {
//
//        let consumer_message_tx = consumer_future.message_tx();
//
//        Pipe {
//            consumer_future,
//        }
//    }
//}
//
//impl<T> Future for Pipe<T>
//    where T: Consumer + Future<Item=(), Error=io::Error> + Send + 'static,
//{
//    type Item = ();
//    type Error = i32;
//
//    fn poll(&mut self) -> Poll<(), i32> {
//        Ok(Async::Ready(()))
//        //Ok(Async::NotReady)
//    }
//}


fn main() {

    //let file_writer = tokio::fs::File::create("foo.txt");
    //let consumer = WriteAdapter::new(file_writer);
    //println!("{:?}", consumer);
    ////let consumer = consumer.map_err(|_| {});

    //let producer = DummyProducer::new();

    ////let pipe = Pipe::new(producer, consumer).map_err(|e| {
    ////    eprintln!("{:?}", e);
    ////});

    ////tokio::run(pipe);

    tokio::run(lazy(|| {

        let file_writer = tokio::fs::File::create("foo.txt");
        let mut consumer = WriteAdapter::new(file_writer);

        let mut count = 0;

        let data = vec![
            vec![65, 66, 67],
            vec![68, 69, 70],
            vec![71, 72, 73, 78],
        ];

        let event_stream = consumer.event_stream().expect("no event stream");

        tokio::spawn(event_stream.for_each(move |event| {
            if count < data.len() {
                consumer.write(data[count].clone());
                count += 1;
            }
            else {
                consumer.end();
            }
            Ok(())
        })
        .map_err(|e| {
            println!("error {:?}", e);
        }));

        Ok(())
    }));
}
