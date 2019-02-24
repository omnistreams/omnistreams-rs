use super::{
    EventEmitter, Transport, Producer, ProducerEventRx, ProducerMessage, ProducerMessageTx,
    ProducerEvent, ProducerEventTx, ProducerMessageRx,
};
use tokio::io;
use tokio::prelude::*;
use futures::sync::mpsc;
use std::collections::HashMap;

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
type EventTx = mpsc::UnboundedSender<Message>;
type Id = u8;


pub struct Multiplexer {
    receivers_rx: Option<mpsc::UnboundedReceiver<Receiver>>,
}

struct InnerTask {
    message_rx: MessageRx,
    receivers_tx: mpsc::UnboundedSender<Receiver>,
    receiver_channels: HashMap<Id, (ProducerMessageRx, ProducerEventTx<Message>)>,
    next_stream_id: Id,
}

pub struct Receiver {
    message_tx: ProducerMessageTx,
    event_rx: Option<ProducerEventRx<Message>>,
}

impl Producer<Message> for Receiver {
    fn request(&mut self, num_items: usize) {
        //self.producer.request(num_items);
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<Message>> {
        Option::take(&mut self.event_rx)
    }
}

impl Multiplexer {
    pub fn new<T: Transport>(mut transport: T) -> Multiplexer {

        let message_rx = transport.messages().expect("messages");
        let (receivers_tx, receivers_rx) = mpsc::unbounded::<Receiver>();

        let inner = InnerTask {
            message_rx,
            receivers_tx,
            receiver_channels: HashMap::new(),
            next_stream_id: 0,
        };

        tokio::spawn(inner.map_err(|_| {}));

        Multiplexer {
            receivers_rx: Some(receivers_rx),
        }
    }

}

impl EventEmitter<Receiver> for Multiplexer {
    fn events(&mut self) -> Option<mpsc::UnboundedReceiver<Receiver>> {
        Option::take(&mut self.receivers_rx)
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
                let id = self.next_stream_id();
                let (message_tx, message_rx) = mpsc::unbounded::<ProducerMessage>();
                let (event_tx, event_rx) = mpsc::unbounded::<ProducerEvent<Message>>();

                let receiver = Receiver {
                    message_tx,
                    event_rx: Some(event_rx),
                };

                self.receiver_channels.insert(id, (message_rx, event_tx));
                self.receivers_tx.unbounded_send(receiver);
            },
            StreamData => {
                println!("StreamData");
                let (_, event_tx) = self.receiver_channels.get(&stream_id).expect("invalid stream id");
                event_tx.unbounded_send(ProducerEvent::Data(data.to_vec()));
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

    fn next_stream_id(&mut self) -> Id {
        let id = self.next_stream_id;
        self.next_stream_id += 1;
        if self.next_stream_id == 255 {
            panic!("out of stream ids");
        }
        id
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
