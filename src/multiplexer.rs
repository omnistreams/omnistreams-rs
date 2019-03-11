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

enum MultiplexerMessage {
    SendControlMessage(Message),
    //CreateConduit,
}

pub enum MultiplexerEvent<P: Producer<Message>> {
    Conduit(P),
    ControlMessage(Message),
}

type Message = Vec<u8>;
type MessageRx = mpsc::UnboundedReceiver<Message>;
//type EventTx = mpsc::UnboundedSender<Message>;
type Id = u8;
type MultiplexerEventTx = mpsc::UnboundedSender<MultiplexerEvent<Receiver>>;
type MultiplexerEventRx = mpsc::UnboundedReceiver<MultiplexerEvent<Receiver>>;


pub struct Multiplexer {
    event_rx: Option<MultiplexerEventRx>,
    message_tx: mpsc::UnboundedSender<MultiplexerMessage>,
}

struct InnerTask<T> 
    where T: Transport + Send,
{
    transport: T,
    transport_done: bool,
    transport_message_rx: MessageRx,
    event_tx: MultiplexerEventTx,
    receiver_channels: HashMap<Id, (ProducerMessageRx, ProducerEventTx<Message>)>,
    next_stream_id: Id,
    message_rx: mpsc::UnboundedReceiver<MultiplexerMessage>,
}

pub struct Receiver {
    message_tx: ProducerMessageTx,
    event_rx: Option<ProducerEventRx<Message>>,
}

impl Producer<Message> for Receiver {
    fn request(&mut self, num_items: usize) {
        self.message_tx.unbounded_send(ProducerMessage::Request(num_items)).unwrap();
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<Message>> {
        Option::take(&mut self.event_rx)
    }
}

impl Multiplexer {
    pub fn new<T: Transport + Send + 'static>(mut transport: T) -> Multiplexer {

        let transport_message_rx = transport.messages().expect("Multiplexer new messages");
        let (message_tx, message_rx) = mpsc::unbounded::<MultiplexerMessage>();
        let (event_tx, event_rx) = mpsc::unbounded();

        let inner = InnerTask {
            transport,
            transport_done: false,
            transport_message_rx,
            event_tx,
            receiver_channels: HashMap::new(),
            next_stream_id: 0,
            message_rx,
        };

        tokio::spawn(inner.map_err(|_| {}));

        Multiplexer {
            event_rx: Some(event_rx),
            message_tx,
        }
    }

    pub fn send_control_message(&mut self, message: Message) {
        self.message_tx.unbounded_send(MultiplexerMessage::SendControlMessage(message))
            .expect("Multiplexer.send_control_message");
    }
}

impl EventEmitter<MultiplexerEvent<Receiver>> for Multiplexer {
    fn events(&mut self) -> Option<MultiplexerEventRx> {
        Option::take(&mut self.event_rx)
    }
}

impl<T> InnerTask<T> 
    where T: Transport + Send,
{
    fn process_messages(&mut self) {
        loop {
            match self.message_rx.poll().unwrap() {
                Async::Ready(Some(message)) => {
                    match message {
                        MultiplexerMessage::SendControlMessage(control_message) => {
                            println!("control message: {:?}", control_message);
                            let mut message = vec![ControlMessage as u8];
                            message.extend(control_message);
                            self.transport.send(message);
                        }
                        //MultiplexerMessage::CreateConduit => {
                        //    println!("create conduit");
                        //}
                    }
                },
                Async::Ready(None) => {
                    break;
                },
                Async::NotReady => {
                    break;
                },
            }
        }
    }

    fn process_transport_messages(&mut self) {

        if self.transport_done {
            return;
        }

        loop {
            match self.transport_message_rx.poll().unwrap() {
                Async::Ready(message) => {
                    match message {
                        Some(m) => {
                            self.handle_message(&m);
                        },
                        None => {
                            self.transport_done = true;
                            break;
                        }
                    }
                },
                Async::NotReady => {
                    break;
                },
            }
        }
    }

    fn process_receiver_messages(&mut self) {
        for (stream_id, (transport_message_rx, _event_tx)) in self.receiver_channels.iter_mut() {
            loop {
                match transport_message_rx.poll().unwrap() {
                    Async::Ready(Some(message)) => {
                        match message {
                            ProducerMessage::Request(num_items) => {
                                let wire_message = vec![StreamRequestData as u8, *stream_id, num_items as u8];
                                self.transport.send(wire_message);
                            }
                        }
                    },
                    Async::Ready(None) => {
                        break;
                    },
                    Async::NotReady => {
                        break;
                    },
                }
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
                let (message_tx, transport_message_rx) = mpsc::unbounded::<ProducerMessage>();
                let (event_tx, event_rx) = mpsc::unbounded::<ProducerEvent<Message>>();

                let receiver = Receiver {
                    message_tx,
                    event_rx: Some(event_rx),
                };

                self.receiver_channels.insert(id, (transport_message_rx, event_tx));
                self.event_tx.unbounded_send(MultiplexerEvent::Conduit(receiver)).unwrap();
            },
            StreamData => {
                //println!("StreamData");
                let (_, event_tx) = self.receiver_channels.get(&stream_id).expect("invalid stream id");
                event_tx.unbounded_send(ProducerEvent::Data(data.to_vec())).unwrap();
            },
            StreamEnd => {
                println!("StreamEnd");
                let (_, event_tx) = self.receiver_channels.get(&stream_id).expect("invalid stream id");
                event_tx.unbounded_send(ProducerEvent::End).unwrap();
            },
            TerminateSender => {
                println!("TerminateSender");
            },
            StreamRequestData => {
                println!("StreamRequestData");
            },
            ControlMessage => {
                println!("ControlMessage");
                self.event_tx.unbounded_send(MultiplexerEvent::ControlMessage(data.to_vec())).unwrap();
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

impl<T> Future for InnerTask<T>
    where T: Transport + Send,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.process_messages();
        self.process_transport_messages();
        self.process_receiver_messages();

        if self.transport_done && self.receiver_channels.len() == 0 {
            println!("ready");
            Ok(Async::Ready(()))
        }
        else {
            Ok(Async::NotReady)
        }
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
            5 => ControlMessage,
            _ => {
                panic!("Invalid message type: {}", val);
            }
        }
    }
}



#[cfg(test)]
mod tests {

    use futures::future::lazy;
    use super::*;

    struct TestTransport {
        message_rx: Option<MessageRx>,
    }

    impl TestTransport {
        fn new() -> TestTransport {

            let (_message_tx, message_rx) = mpsc::unbounded::<Message>();

            TestTransport {
                message_rx: Some(message_rx),
            }
        }
    }

    impl Transport for TestTransport {
        fn send(&mut self, _message: Message) {
            //self.socket.send(message);
        }

        fn messages(&mut self) -> Option<MessageRx> {
            Option::take(&mut self.message_rx)
        }
    }

    #[test]
    fn create() {
        tokio::run(lazy(|| {
            Multiplexer::new(TestTransport::new());
            Ok(())
        }));
    }

    #[test]
    fn transfer_largefile() {
    }
}
