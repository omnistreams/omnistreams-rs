use futures::sync::mpsc;

//mod read_adapter;
mod write_adapter;
mod sink_adapter;
mod map_conduit;
mod range_producer;
mod transport;
mod multiplexer;
mod producer;
mod consumer;

pub mod runtime {
    use futures::future::lazy;
    pub fn run(f: fn()) {
        tokio::run(lazy(move || {
            f();
            Ok(())
        }));
    }
}

//pub use self::read_adapter::ReadAdapter;
pub use self::write_adapter::WriteAdapter;
pub use self::sink_adapter::SinkAdapter;
pub use self::range_producer::{RangeProducer, RangeProducerBuilder};
pub use self::map_conduit::{MapConduit, MapConsumer, MapProducer};
//pub use self::transport::{Transport, Acceptor, WebSocketTransport, WebSocketAcceptorBuilder};
pub use self::transport::{Transport};
pub use self::multiplexer::{Multiplexer, MultiplexerEvent};
pub use self::producer::{
    Producer, ProducerEvent, ProducerEventRx, ProducerEventTx,
    ProducerMessage, ProducerMessageRx, ProducerMessageTx,
    ProducerEventEmitter,
};

pub use self::consumer::{
    Consumer, ConsumerEvent, ConsumerEventRx, ConsumerEventTx,
    ConsumerMessage, ConsumerMessageRx, ConsumerMessageTx,
};

pub type Message = Vec<u8>;

#[derive(PartialEq, Clone, Debug)]
pub enum CancelReason {
    Disconnected,
    Other(String),
}

pub trait Streamer {
    fn cancel(&mut self, reason: CancelReason);
}

pub trait Conduit<A, B> : Consumer<A> + Producer<B>
    where B: Send + 'static
{
    
    type ConcreteConsumer: Consumer<A>;
    type ConcreteProducer: Producer<B>;

    fn split(self) -> (Self::ConcreteConsumer, Self::ConcreteProducer);
}

// TODO: maybe add a default method for taking the events object
pub trait EventEmitter<T> {
    fn events(&mut self) -> Option<mpsc::UnboundedReceiver<T>>;
}
