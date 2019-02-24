use futures::sync::mpsc;
use std::thread;

type Message = Vec<u8>;
type MessageRx = mpsc::UnboundedReceiver<Message>;
type WebSocketTransportRx = mpsc::UnboundedReceiver<WebSocketTransport>;


pub trait Transport {
}

pub trait Acceptor {
    fn transports(&mut self) -> Option<WebSocketTransportRx>;
}

#[derive(Debug)]
pub struct WebSocketTransport {
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
    pub fn new(socket: ws::Sender, rx: MessageRx) -> WebSocketTransport {
        WebSocketTransport {
        }
    }
}

impl Transport for WebSocketTransport {
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

        let (tx, rx) = mpsc::unbounded::<WebSocketTransport>();

        println!("{}", addr);

        thread::spawn(move || {
            ws::listen(addr, |socket| {

                let (msg_in_tx, msg_in_rx) = mpsc::unbounded::<Message>();

                let transport = WebSocketTransport::new(socket, msg_in_rx);

                tx.unbounded_send(transport);

                move |msg| {
                    match msg {
                        ws::Message::Text(_) => {
                            panic!("text ws message");
                        },
                        ws::Message::Binary(m) => {
                            msg_in_tx.unbounded_send(m);
                        },
                    }
                    Ok(())
                }
            });
        });

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
    use super::*;

    #[test]
    fn create() {
        WebSocketTransport::new();
    }
    
    #[test]
    fn create_ws_acceptor() {
        WebSocketAcceptor::new();
    }

    #[test]
    fn create_ws_initiator() {
        WebSocketInitiator::new();
    }
}
