fn main() {
}
//use omnistreams::{Producer, ProducerEvent, Multiplexer, Transport, EventEmitter};
//use futures::future::lazy;
//use tokio::prelude::*;
//use futures::sync::mpsc;
//use std::time::{Instant};
//
//
//type Message = Vec<u8>;
//type MessageTx = mpsc::UnboundedSender<Message>;
//type MessageRx = mpsc::UnboundedReceiver<Message>;
//
//struct TestTransport {
//    message_tx: mpsc::UnboundedSender<Message>,
//    message_rx: Option<MessageRx>,
//}
//
//impl TestTransport {
//    fn new() -> (TestTransport, MessageTx, MessageRx) {
//
//        let (in_tx, in_rx) = mpsc::unbounded::<Message>();
//        let (out_tx, out_rx) = mpsc::unbounded::<Message>();
//
//        (TestTransport {
//            message_tx: out_tx,
//            message_rx: Some(in_rx),
//        }, in_tx, out_rx)
//    }
//}
//
//impl Transport for TestTransport {
//    fn send(&mut self, message: Message) {
//        (&self.message_tx).unbounded_send(message).unwrap();
//    }
//
//    fn messages(&mut self) -> Option<MessageRx> {
//        Option::take(&mut self.message_rx)
//    }
//}
//
//fn main() {
//
//    tokio::run(lazy(|| {
//        let (transport, tx, rx) = TestTransport::new();
//        let mut mux = Multiplexer::new(transport);
//
//        let events = mux.events().unwrap();
//
//        let start = Instant::now();
//
//        tokio::spawn(events.for_each(move |mut producer| {
//            println!("got prod");
//
//            let events = producer.events().unwrap();
//
//            //producer.request(255);
//
//            let mut bytes_received = 0;
//
//            tokio::spawn(events.for_each(move |event| {
//                //println!("receiver event: {:?}", event);
//                match event {
//                    ProducerEvent::Data(data) => {
//                        bytes_received += data.len();
//                        producer.request(1);
//                    },
//                    ProducerEvent::End => {
//                        let sec = start.elapsed().as_micros() as f64 / 1000_000.0;
//                        println!("Time: {}", sec);
//                        println!("Bytes Received: {}", bytes_received);
//                        println!("Bitrate: {} Mbps", bytes_received as f64 / 1024.0 / 1024.0 * 8.0 / sec);
//                    },
//                }
//                Ok(())
//            })
//            .map_err(|e| {
//                eprintln!("{:?}", e);
//            }));
//
//            Ok(())
//        })
//        .map_err(|e| {
//            eprintln!("{:?}", e);
//        }));
//
//        tx.unbounded_send(vec![0, 0]).unwrap();
//        tx.unbounded_send(vec![1, 0, 65]).unwrap();
//
//        let chunk_size = 1024*1024;
//        let mut data = vec![65; chunk_size];
//        data[0] = 1;
//        data[1] = 0;
//        let num_chunks = 1_000;
//        let mut index = 0;
//        let mut ended = false;
//
//        println!("len: {}", data.len());
//
//        tokio::spawn(rx.for_each(move |_message| {
//            //println!("{:?}", message);
//
//            if index < num_chunks {
//                // stream data
//                tx.unbounded_send(data.clone()).unwrap();
//                index += 1;
//            }
//            else if !ended {
//                ended = true;
//                // end
//                tx.unbounded_send(vec![2, 0]).unwrap();
//            }
//            Ok(())
//        })
//        .map_err(|e| {
//            eprintln!("{:?}", e);
//        }));
//
//        Ok(())
//    }));
//}
