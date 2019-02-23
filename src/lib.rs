use futures::sync::mpsc;
use tokio::prelude::*;

mod read_adapter;
mod write_adapter;

pub use self::read_adapter::ReadAdapter;
pub use self::write_adapter::WriteAdapter;


type MessageRx<T> = mpsc::UnboundedReceiver<ConsumerMessage<T>>;
type MessageTx<T> = mpsc::UnboundedSender<ConsumerMessage<T>>;

type EventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type EventTx = mpsc::UnboundedSender<ConsumerEvent>;

type ProducerEventRx<T> = mpsc::UnboundedReceiver<ProducerEvent<T>>;
type ProducerEventTx<T> = mpsc::UnboundedSender<ProducerEvent<T>>;

type ProducerMessageRx = mpsc::UnboundedReceiver<ProducerMessage>;
type ProducerMessageTx = mpsc::UnboundedSender<ProducerMessage>;


pub trait Consumer<T> {
    fn write(&mut self, data: T);
    fn end(&mut self);
    fn event_stream(&mut self) -> Option<EventRx>;
}

pub trait Producer<T> {
    fn request(&mut self, num_items: usize);
    fn event_stream(&mut self) -> Option<ProducerEventRx<T>>;
    fn pipe<C>(self, consumer: C)
        where Self: Sized + Send + 'static,
              C: Consumer<T> + Sized + Send + 'static,
              T: Send + 'static,
    {
        pipe(self, consumer);
    }
}

#[derive(Debug)]
pub enum ConsumerMessage<T> {
    Write(T),
    End,
}

#[derive(Debug)]
pub enum ProducerMessage {
    Request(usize),
}

#[derive(Debug)]
pub enum ConsumerEvent {
    Request(usize),
}

#[derive(Debug)]
pub enum ProducerEvent<T> {
    Data(T),
    End,
}

pub fn pipe<T, P, C>(mut producer: P, mut consumer: C)
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
                println!("producer ended");
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
        }
        
        Ok(())
    })
    .map_err(|e| {
        println!("error {:?}", e);
    }));
}
