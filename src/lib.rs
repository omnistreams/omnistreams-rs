use futures::sync::mpsc;

mod read_adapter;
mod write_adapter;
mod sink_adapter;
mod map_conduit;
mod range_producer;
mod transport;
mod multiplexer;
mod producer;

pub mod runtime {
    use futures::future::lazy;
    pub fn run(f: fn()) {
        tokio::run(lazy(move || {
            f();
            Ok(())
        }));
    }
}

pub use self::read_adapter::ReadAdapter;
pub use self::write_adapter::WriteAdapter;
pub use self::sink_adapter::SinkAdapter;
pub use self::range_producer::{RangeProducer, RangeProducerBuilder};
pub use self::map_conduit::{MapConduit, MapConsumer, MapProducer};
pub use self::transport::{Transport, Acceptor, WebSocketTransport, WebSocketAcceptorBuilder};
pub use self::multiplexer::{Multiplexer, MultiplexerEvent};
pub use self::producer::{
    Producer, ProducerEvent, ProducerEventRx, ProducerEventTx,
    ProducerMessage, ProducerMessageRx, ProducerMessageTx,
};

pub type Message = Vec<u8>;

type ConsumerMessageRx<T> = mpsc::UnboundedReceiver<ConsumerMessage<T>>;
type ConsumerMessageTx<T> = mpsc::UnboundedSender<ConsumerMessage<T>>;

pub type ConsumerEventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type ConsumerEventTx = mpsc::UnboundedSender<ConsumerEvent>;


// TODO: maybe add a default method for taking the events object
pub trait EventEmitter<T> {
    fn events(&mut self) -> Option<mpsc::UnboundedReceiver<T>>;
}

pub trait Streamer {
    fn cancel(&mut self, reason: CancelReason);
}

pub trait Consumer<T> {
    fn write(&self, data: T);
    fn end(&self);
    fn event_stream(&mut self) -> Option<ConsumerEventRx>;
}

pub trait Conduit<A, B> : Consumer<A> + Producer<B> {
    
    type ConcreteConsumer: Consumer<A>;
    type ConcreteProducer: Producer<B>;

    fn split(self) -> (Self::ConcreteConsumer, Self::ConcreteProducer);
}

#[derive(Debug)]
pub enum ConsumerMessage<T> {
    Write(T),
    End,
}



#[derive(PartialEq, Clone, Debug)]
pub enum ConsumerEvent {
    Request(usize),
    Cancellation(CancelReason),
}



#[derive(PartialEq, Clone, Debug)]
pub enum CancelReason {
    Disconnected,
    Other(String),
}
