use omnistreams::{Consumer, WriteAdapter};
use tokio::prelude::*;
use futures::future::lazy;


fn main() {

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

        tokio::spawn(event_stream.for_each(move |_event| {
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
