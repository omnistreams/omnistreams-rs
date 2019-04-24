use futures::sync::mpsc;
//use websocket::r#async::Server;
//use websocket::server::InvalidConnection;
//use websocket::message::{OwnedMessage};
//use websocket::result::WebSocketError;
//use tokio::reactor::Handle;
//use futures::stream::{Stream};
//use futures::sink::{Sink};
//use futures::future::{Future};
//use std::fmt::Debug;
use tokio::prelude::*;

type Message = Vec<u8>;
type MessageRx = mpsc::UnboundedReceiver<Message>;
//type MessageTx = mpsc::UnboundedSender<Message>;
//type WebSocketTransportRx = mpsc::UnboundedReceiver<WebSocketTransport>;

pub type TransportEventRx = mpsc::UnboundedReceiver<TransportEvent>;
type TransportEventTx = mpsc::UnboundedSender<TransportEvent>;

pub trait Transport {
    fn send(&mut self, message: Message);
    //fn messages(&mut self) -> Option<MessageRx>;
    fn event_stream(&mut self) -> Option<TransportEventRx>;

    fn events(&mut self) -> TransportEventEmitter {
        let event_rx = self.event_stream().expect("transport event stream");
        TransportEventEmitter::new(event_rx)
    }
}

#[derive(Debug)]
pub enum TransportEvent {
    Message(Message),
    Close,
}

pub struct TransportEventEmitter {
    event_rx: Option<TransportEventRx>,
}

impl TransportEventEmitter {
    pub fn new(event_rx: TransportEventRx) -> Self {
        Self {
            event_rx: Some(event_rx),
        }
    }

    pub fn for_each<C: FnMut(TransportEvent) + Send + 'static>(&mut self, mut callback: C) {
        let rx = Option::take(&mut self.event_rx).expect("transport take event_rx");

        tokio::spawn(rx.for_each(move |event| {
            callback(event);
            Ok(())
        }));
    }
}

//pub trait Acceptor {
//    fn transports(&mut self) -> Option<WebSocketTransportRx>;
//}
//
//pub struct WebSocketTransport {
//    in_rx: Option<MessageRx>,
//    out_tx: MessageTx,
//}
//
//pub struct WebSocketAcceptor {
//    stream: Option<WebSocketTransportRx>,
//}
//
//pub struct WebSocketAcceptorBuilder {
//    host: String,
//    port: u16,
//}
//
////pub struct WebSocketInitiator {
////}
//
//impl WebSocketTransport {
//    //pub fn new(rx: MessageRx, sink: SplitSink<OwnedMessage>) -> WebSocketTransport {
//    //    WebSocketTransport {
//    //        sink,
//    //        message_rx: Some(rx),
//    //    }
//    //}
//}
//
//impl Transport for WebSocketTransport {
//
//    fn send(&mut self, message: Message) {
//        self.out_tx.unbounded_send(message).expect("ws transport send");
//    }
//
//    fn messages(&mut self) -> Option<MessageRx> {
//        Option::take(&mut self.in_rx)
//    }
//}
//
//impl Acceptor for WebSocketAcceptor {
//    fn transports(&mut self) -> Option<WebSocketTransportRx> {
//        Option::take(&mut self.stream)
//    }
//}
//
//
//impl WebSocketAcceptorBuilder {
//    pub fn new() -> WebSocketAcceptorBuilder {
//
//        WebSocketAcceptorBuilder {
//            host: "127.0.0.1".to_string(),
//            port: 8080,
//        }
//    }
//
//    pub fn host(mut self, value: &str) -> WebSocketAcceptorBuilder {
//        self.host = value.to_string();
//        self
//    }
//
//    pub fn port(mut self, value: u16) -> WebSocketAcceptorBuilder {
//        self.port = value;
//        self
//    }
//
//    pub fn build(self) -> WebSocketAcceptor {
//        let addr = format!("{}:{}", self.host, self.port);
//        println!("{}", addr);
//
//        let (tx, rx) = mpsc::unbounded::<WebSocketTransport>();
//
//        let server = Server::bind(addr, &Handle::default()).expect("ws server bind");
//
//        let f = server
//            .incoming()
//            // we don't wanna save the stream if it drops
//            .map_err(|InvalidConnection { error, .. }| error)
//            .for_each(move |(upgrade, addr)| {
//                println!("Got a connection from: {}", addr);
//
//                let tx = tx.clone();
//
//                // accept the request to be a ws connection if it does
//                let f = upgrade
//                    .accept()
//                    .and_then(move |(s, _)| {
//                        let (sink, stream) = s.split();
//                        let (in_tx, in_rx) = mpsc::unbounded::<Message>();
//                        let (out_tx, out_rx) = mpsc::unbounded::<Message>();
//
//                        let sink_f = sink.send_all(out_rx.map_err(|_e| {
//                            WebSocketError::NoDataAvailable
//                        })
//                        .map(|m| {
//                            OwnedMessage::Binary(m)
//                        }));
//
//                        spawn_future(sink_f, "Sink Fut");
//
//                        let transport = WebSocketTransport {
//                            out_tx,
//                            in_rx: Some(in_rx),
//                        };
//
//                        tx.unbounded_send(transport).expect("send transport");
//
//                        stream
//                            .take_while(|m| Ok(!m.is_close()))
//                            .filter_map(|m| {
//                                match m {
//                                    OwnedMessage::Ping(_p) => {
//                                        //Some(OwnedMessage::Pong(p))
//                                        None
//                                    },
//                                    OwnedMessage::Pong(_) => None,
//                                    OwnedMessage::Binary(bm) => {
//                                        Some(bm)
//                                    },
//                                    _ => {
//                                        //Some(m)
//                                        None
//                                    },
//                                }
//                            })
//                            // TODO: This should be possible using Stream.forward
//                            //.forward(in_tx)
//                            .for_each(move |m| {
//                                in_tx.unbounded_send(m).expect("send");
//                                Ok(())
//                            })
//                            // TODO: properly close
//                            //.and_then(|(_, sink)| sink.send(OwnedMessage::Close(None)))
//                    });
//
//                spawn_future(f, "Client Status");
//
//                Ok(())
//        });
//
//        spawn_future(f, "Server fut");
//
//        WebSocketAcceptor {
//            stream: Some(rx),
//        }
//    }
//}
//
//fn spawn_future<F, I, E>(f: F, desc: &'static str)
//where
//	F: Future<Item = I, Error = E> + 'static + Send,
//	E: Debug,
//{
//	tokio::spawn(
//		f.map_err(move |e| println!("{}: '{:?}'", desc, e))
//			.map(move |_| println!("{}: Finished.", desc)),
//	);
//}
//
////impl WebSocketInitiator {
////    pub fn new() -> WebSocketInitiator {
////        WebSocketInitiator {
////        }
////    }
////}
//
//#[cfg(test)]
//mod tests {
//
//    //use futures::future::lazy;
//    //use super::*;
//
//    //#[test]
//    //fn create() {
//    //    WebSocketTransport::new();
//    //}
//    //
//    //#[test]
//    //fn create_ws_acceptor() {
//    //    WebSocketAcceptor::new();
//    //}
//
//    //#[test]
//    //fn create_ws_initiator() {
//    //    WebSocketInitiator::new();
//    //}
//}
