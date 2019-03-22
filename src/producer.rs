use futures::sync::mpsc;
use tokio::prelude::*;

use super::{CancelReason, Streamer, Consumer, ConsumerEvent, Conduit};

pub type ProducerEventRx<T> = mpsc::UnboundedReceiver<ProducerEvent<T>>;
pub type ProducerEventTx<T> = mpsc::UnboundedSender<ProducerEvent<T>>;

pub type ProducerMessageRx = mpsc::UnboundedReceiver<ProducerMessage>;
pub type ProducerMessageTx = mpsc::UnboundedSender<ProducerMessage>;


#[derive(Debug)]
pub enum ProducerMessage {
    Request(usize),
    Cancel(CancelReason),
}

#[derive(Debug)]
pub enum ProducerEvent<T> {
    Data(T),
    End,
}

pub trait Producer<T> : Streamer
    where T: Send + 'static
{
    fn request(&mut self, num_items: usize);
    fn event_stream(&mut self) -> Option<ProducerEventRx<T>>;
    //fn events(&mut self) -> ProducerEventEmitter<T>;
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

    fn events(&mut self) -> ProducerEventEmitter<T> {

        let event_rx = self.event_stream().expect("event stream");
            
        ProducerEventEmitter::new(event_rx)
    }
}

pub struct ProducerEventEmitter<T>
    where T: Send + 'static
{
    event_rx: Option<ProducerEventRx<T>>,
}

impl<T> ProducerEventEmitter<T>
    where T: Send + 'static
{
    pub fn new(event_rx: ProducerEventRx<T>) -> Self {
        Self {
            event_rx: Some(event_rx),
        }
    }

    pub fn for_each<C: FnMut(ProducerEvent<T>) + Send + 'static>(&mut self, mut callback: C) {
        let rx = Option::take(&mut self.event_rx).expect("take event_rx");

        tokio::spawn(rx.for_each(move |event| {
            callback(event);
            Ok(())
        }));
    }
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
