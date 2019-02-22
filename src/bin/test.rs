use omnistreams::{
    Producer, ProducerEvent, Consumer, ConsumerEvent, ReadAdapter,
    WriteAdapter
};
use tokio::prelude::*;
use futures::future::lazy;
use std::str;


fn main() {

    tokio::run(lazy(|| {

        let file_reader = tokio::fs::File::open("in.txt");
        let mut producer = ReadAdapter::new(file_reader);
        let producer_events = producer.event_stream().unwrap();

        let file_writer = tokio::fs::File::create("out.txt");
        let mut consumer = WriteAdapter::new(file_writer);
        let consumer_events = consumer.event_stream().expect("no event stream");

        tokio::spawn(producer_events.for_each(move |event| {
            match event {
                ProducerEvent::Data(data) => {
                    println!("{:?}", str::from_utf8(&data).unwrap());
                    consumer.write(data);
                },
                ProducerEvent::End => {
                    println!("producer ended");
                },
            }
            Ok(())
        })
        .map_err(|e| {
            println!("error {:?}", e);
        }));

        tokio::spawn(consumer_events.for_each(move |event| {
            match event {
                ConsumerEvent::Request(num_items) => {
                    producer.request(num_items);
                },
            }
            
            Ok(())
        })
        .map_err(|e| {
            println!("error {:?}", e);
        }));

        Ok(())
    }));
}
