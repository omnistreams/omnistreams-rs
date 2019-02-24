use super::{Transport, Producer, ProducerEventRx};
use tokio::io;
use tokio::prelude::*;
use futures::sync::mpsc;

use self::MessageType::*;

enum MessageType {
    CreateReceiver = 0,
    StreamData = 1,
    StreamEnd = 2,
    TerminateSender = 3,
    StreamRequestData = 4,
    ControlMessage = 5,
}

type Message = Vec<u8>;
type MessageRx = mpsc::UnboundedReceiver<Message>;


pub struct Multiplexer {
}

struct InnerTask {
    message_rx: MessageRx,
}

struct Receiver<T> {
    message_rx: Option<ProducerEventRx<T>>,
}

impl<T> Producer<T> for Receiver<T> {
    fn request(&mut self, num_items: usize) {
        //self.producer.request(num_items);
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<T>> {
        Option::take(&mut self.message_rx)
    }
}

impl Multiplexer {
    pub fn new<T: Transport>(mut transport: T) -> Multiplexer {

        let message_rx = transport.messages().expect("messages");

        let inner = InnerTask {
            message_rx,
        };

        tokio::spawn(inner.map_err(|_| {}));

        Multiplexer {
        }
    }
}

impl InnerTask {
    fn process_transport_messages(&mut self) {
        loop {
            match self.message_rx.poll().unwrap() {
                Async::Ready(message) => {
                    println!("Message: {:?}", message);
                    self.handle_message(&message.expect("message"));
                },
                Async::NotReady => {
                    break;
                },
            }
        }
    }

    fn handle_message(&mut self, message: &[u8]) {

        let message_type: MessageType = message[0].into();
        let stream_id = message[1];
        let data = &message[2..];

        match message_type {
            CreateReceiver => {
                println!("CreateReceiver: {}", stream_id);
            },
            StreamData => {
                println!("StreamData");
                println!("{:?}", data);
            },
            StreamEnd => {
                println!("StreamEnd");
            },
            TerminateSender => {
                println!("TerminateSender");
            },
            StreamRequestData => {
                println!("StreamRequestData");
            },
            ControlMessage => {
                println!("ControlMessage");
            },
        }
    }

}

impl Future for InnerTask {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.process_transport_messages();
        //Ok(Async::Ready(()))
        Ok(Async::NotReady)
    }
}


impl From<u8> for MessageType {
    fn from(val: u8) -> MessageType {
        match val {
            0 => CreateReceiver,
            1 => StreamData,
            2 => StreamEnd,
            3 => TerminateSender,
            4 => StreamRequestData,
            6 => ControlMessage,
            _ => {
                panic!("Invalid message type");
            }
        }
    }
}



#[cfg(test)]
mod tests {

    use futures::future::lazy;
    use super::*;

    struct TestTransport {
    }

    impl TestTransport {
        fn new() -> TestTransport {
            TestTransport {
            }
        }
    }

    impl Transport for TestTransport {
    }

    #[test]
    fn create() {
        tokio::run(lazy(|| {
            Multiplexer::new(TestTransport::new());
            Ok(())
        }));
    }
}
