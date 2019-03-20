use futures::sync::mpsc;
use tokio::prelude::*;

mod read_adapter;
mod write_adapter;
mod sink_adapter;
mod map_conduit;
mod range_producer;
mod transport;
mod multiplexer;

pub use self::read_adapter::ReadAdapter;
pub use self::write_adapter::WriteAdapter;
pub use self::sink_adapter::SinkAdapter;
pub use self::range_producer::{RangeProducer, RangeProducerBuilder};
pub use self::map_conduit::MapConduit;
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

pub type ConduitConsumer<T> = Box<dyn Consumer<T> + Send>;
pub type ConduitProducer<T> = Box<dyn Producer<T> + Send>;


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
    fn pipe_into(self, consumer: Box<dyn Consumer<T> + Send>)
        where Self: Sized + Send + 'static,
              T: Send + 'static,
    {
        pipe(Box::new(self), consumer);
    }

    fn pipe_through<C, U>(self, conduit: C) -> ConduitProducer<U>
        where Self: Sized + Send + 'static,
              C: Conduit<T, U> + Sized + Send + 'static,
              T: Send + 'static,
              U: Send + 'static,
    {
        let (consumer, producer) = conduit.split();
        pipe(Box::new(self), consumer);

        producer
    }
}

pub trait Conduit<A, B> : Consumer<A> + Producer<B> {
    fn split(self) -> (ConduitConsumer<A>, ConduitProducer<B>);
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

pub fn pipe<T>(mut producer: Box<dyn Producer<T> + Send>, mut consumer: Box<dyn Consumer<T> + Send>)
    where T: Send + 'static,
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
