use futures::sync::mpsc;
use std::thread;
use websocket::r#async::Server;
use websocket::server::InvalidConnection;
use websocket::message::{OwnedMessage};
use tokio::reactor::Handle;
use tokio::codec::{Framed};
use tokio::net::TcpStream;
use websocket::codec::ws::MessageCodec;
use futures::stream::{Stream, SplitSink};
use futures::sink::{Sink};
use futures::future::{Future};

type Message = Vec<u8>;
type MessageRx = mpsc::UnboundedReceiver<Message>;
type WebSocketTransportRx = mpsc::UnboundedReceiver<WebSocketTransport>;


pub trait Transport {
    fn send(&mut self, message: Message);
    fn messages(&mut self) -> Option<MessageRx>;
}

pub trait Acceptor {
    fn transports(&mut self) -> Option<WebSocketTransportRx>;
}

pub struct WebSocketTransport {
    sink: SplitSink<Framed<TcpStream, MessageCodec<OwnedMessage>>>,
    message_rx: Option<MessageRx>,
}

pub struct WebSocketAcceptor {
    stream: Option<WebSocketTransportRx>,
}

pub struct WebSocketAcceptorBuilder {
    host: String,
    port: u16,
}

pub struct WebSocketInitiator {
}

impl WebSocketTransport {
    //pub fn new(rx: MessageRx, sink: SplitSink<OwnedMessage>) -> WebSocketTransport {
    //    WebSocketTransport {
    //        sink,
    //        message_rx: Some(rx),
    //    }
    //}
}

impl Transport for WebSocketTransport {

    fn send(&mut self, message: Message) {
        //let sink = &self.sink;
        //sink.send(OwnedMessage::Binary(message));
    }

    fn messages(&mut self) -> Option<MessageRx> {
        Option::take(&mut self.message_rx)
    }
}

impl Acceptor for WebSocketAcceptor {
    fn transports(&mut self) -> Option<WebSocketTransportRx> {
        Option::take(&mut self.stream)
    }
}


impl WebSocketAcceptorBuilder {
    pub fn new() -> WebSocketAcceptorBuilder {

        WebSocketAcceptorBuilder {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }

    pub fn host(mut self, value: &str) -> WebSocketAcceptorBuilder {
        self.host = value.to_string();
        self
    }

    pub fn port(mut self, value: u16) -> WebSocketAcceptorBuilder {
        self.port = value;
        self
    }

    pub fn build(self) -> WebSocketAcceptor {
        let addr = format!("{}:{}", self.host, self.port);
        println!("{}", addr);

        let (tx, rx) = mpsc::unbounded::<WebSocketTransport>();

        let server = Server::bind(addr, &Handle::default()).expect("ws server bind");
        let f = server.incoming()
            //.map_err(|InvalidConnection { error, .. }| error)
            .map_err(|_| { 
                println!("ws error");
                ()
            })
            .for_each(move |(upgrade, addr)| {
                println!("got conn");


                let f = upgrade.accept()
                    .and_then(|(s, _)| {

                        println!("accepted");

                        let (message_tx, message_rx) = mpsc::unbounded::<Message>();
                        let (sink, stream) = s.split();

                        //let transport = WebSocketTransport {
                        //    sink,
                        //    message_rx: Some(message_rx),
                        //};

                        stream
                            .take_while(|m| Ok(!m.is_close()))
                            .filter_map(|m| {
                                println!("Message from Client: {:?}", m);
                                match m {
                                    OwnedMessage::Ping(p) => {
                                        println!("ping");
                                        //sink.send(Some(OwnedMessage::Pong(p)))
                                        None
                                    },
                                    OwnedMessage::Pong(_) => {
                                        println!("pong");
                                        None
                                    },
                                    OwnedMessage::Binary(m) => {
                                        println!("bin: {:?}", m);
                                        Some(m)
                                    },
                                    _ => {
                                        println!("other");
                                        None
                                    }
                                }
                            })
                            //.forward(message_tx)
                            ;

                        Ok(())
                    });
                Ok(())
            });

        tokio::spawn(f);

        //thread::spawn(move || {
        //    ws::listen(addr, |socket| {

        //        let (msg_in_tx, msg_in_rx) = mpsc::unbounded::<Message>();

        //        let transport = WebSocketTransport::new(socket, msg_in_rx);

        //        tx.unbounded_send(transport);

        //        move |msg| {
        //            match msg {
        //                ws::Message::Text(_) => {
        //                    panic!("text ws message");
        //                },
        //                ws::Message::Binary(m) => {
        //                    msg_in_tx.unbounded_send(m);
        //                },
        //            }
        //            Ok(())
        //        }
        //    });
        //});

        WebSocketAcceptor {
            stream: Some(rx),
        }
    }
}

impl WebSocketInitiator {
    pub fn new() -> WebSocketInitiator {
        WebSocketInitiator {
        }
    }
}

#[cfg(test)]
mod tests {

    //use futures::future::lazy;
    //use super::*;

    //#[test]
    //fn create() {
    //    WebSocketTransport::new();
    //}
    //
    //#[test]
    //fn create_ws_acceptor() {
    //    WebSocketAcceptor::new();
    //}

    //#[test]
    //fn create_ws_initiator() {
    //    WebSocketInitiator::new();
    //}
}
