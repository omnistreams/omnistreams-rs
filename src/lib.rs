use futures::sync::mpsc;
use tokio::prelude::*;

mod read_adapter;
mod write_adapter;
mod sink_adapter;
mod map_conduit;
mod range_producer;
mod transport;
mod multiplexer;

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

pub type Message = Vec<u8>;

type ConsumerMessageRx<T> = mpsc::UnboundedReceiver<ConsumerMessage<T>>;
type ConsumerMessageTx<T> = mpsc::UnboundedSender<ConsumerMessage<T>>;

pub type ConsumerEventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type ConsumerEventTx = mpsc::UnboundedSender<ConsumerEvent>;

pub type ProducerEventRx<T> = mpsc::UnboundedReceiver<ProducerEvent<T>>;
type ProducerEventTx<T> = mpsc::UnboundedSender<ProducerEvent<T>>;

type ProducerMessageRx = mpsc::UnboundedReceiver<ProducerMessage>;
type ProducerMessageTx = mpsc::UnboundedSender<ProducerMessage>;


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

pub trait Producer<T> : Streamer {
    fn request(&mut self, num_items: usize);
    fn event_stream(&mut self) -> Option<ProducerEventRx<T>>;
    fn pipe_into<C>(self, consumer: C)
        where Self: Sized + Send + 'static,
              C: Consumer<T> + Sized + Send + 'static,
              T: Send + 'static,
    {
        pipe_into(self, consumer);
    }

    fn pipe_through<C, U>(self, conduit: C) -> C::ConcreteProducer
        where Self: Sized + Send + 'static,
              C: Conduit<T, U> + Sized + Send + 'static,
              T: Send + 'static,
              U: Send + 'static,
              C::ConcreteConsumer: Send,
    {
        let (consumer, producer) = conduit.split();
        pipe_into(self, consumer);

        producer
    }
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

#[derive(Debug)]
pub enum ProducerMessage {
    Request(usize),
    Cancel(CancelReason),
}

#[derive(PartialEq, Clone, Debug)]
pub enum ConsumerEvent {
    Request(usize),
    Cancellation(CancelReason),
}

#[derive(Debug)]
pub enum ProducerEvent<T> {
    Data(T),
    End,
}

#[derive(PartialEq, Clone, Debug)]
pub enum CancelReason {
    Disconnected,
    Other(String),
}

pub fn pipe_into<T, P, C>(mut producer: P, mut consumer: C)
    where T: Send + 'static,
          P: Producer<T> + Send + 'static,
          C: Consumer<T> + Send + 'static
{
    let producer_events = producer.event_stream().unwrap();
    let consumer_events = consumer.event_stream().expect("no event stream");

    tokio::spawn(producer_events.for_each(move |event| {
        match event {
            ProducerEvent::Data(data) => {
                consumer.write(data);
            },
            ProducerEvent::End => {
                consumer.end();
            },
        }
        Ok(())
    })
    .map_err(|e| {
        println!("error {:?}", e);
    }));

    tokio::spawn(consumer_events.for_each(move |event| {
        match event {
            ConsumerEvent::Request(num_items) => {
                producer.request(num_items);
            },
            ConsumerEvent::Cancellation(reason) => {
                producer.cancel(reason);
            },
        }
        
        Ok(())
    })
    .map_err(|e| {
        println!("error {:?}", e);
    }));
}
