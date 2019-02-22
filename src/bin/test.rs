use omnistreams::{
    Producer, ProducerEvent, Consumer, ConsumerEvent, ReadAdapter,
    WriteAdapter
};
use tokio::prelude::*;
use futures::future::lazy;


fn main() {

    tokio::run(lazy(|| {

        let file_reader = tokio::fs::File::open("in.txt");
        let mut producer = ReadAdapter::new(file_reader);

        let file_writer = tokio::fs::File::create("out.txt");
        let mut consumer = WriteAdapter::new(file_writer);

        let producer_events = producer.event_stream().unwrap();
        tokio::spawn(producer_events.for_each(|event| {
            match event {
                ProducerEvent::Data(data) => {
                    println!("producer data");
                    //producer.request(1);
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

        //producer.request(1);

        let mut count = 0;

        let data = vec![
            vec![65, 66, 67],
            vec![68, 69, 70],
            vec![71, 72, 73, 78],
        ];

        let event_stream = consumer.event_stream().expect("no event stream");

        tokio::spawn(event_stream.for_each(move |event| {
            match event {
                ConsumerEvent::Request(num_items) => {
                    if count < data.len() {
                        consumer.write(data[count].clone());
                        count += 1;
                    }
                    else {
                        consumer.end();
                    }
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
