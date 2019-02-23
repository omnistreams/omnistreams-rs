use super::{
    Consumer, ConsumerEventRx, Producer, ProducerEventRx, Conduit,
    ConsumerMessage, ConsumerEvent, ProducerMessage, ProducerEvent,
    ConsumerMessageTx, ProducerMessageTx,
    ConsumerMessageRx, ConsumerEventTx, ProducerMessageRx, ProducerEventTx,
};
use std::marker::PhantomData;
use futures::sync::mpsc;
use futures::future::lazy;
use tokio::io;
use tokio::prelude::*;


pub struct MapConduit<F, A, B>
    where F: Fn(A) -> B
{
    f: F,
    in_type: PhantomData<A>,
    out_type: PhantomData<B>,
    consumer: MapConsumer<A>,
    producer: MapProducer<B>,
}

pub struct MapConsumer<A> {
    message_tx: ConsumerMessageTx<A>,
    event_rx: Option<ConsumerEventRx>,
}

pub struct MapProducer<B> {
    message_tx: ProducerMessageTx,
    event_rx: Option<ProducerEventRx<B>>,
}

struct InnerTask<A, B> {
    c_message_rx: ConsumerMessageRx<A>,
    c_event_tx: ConsumerEventTx,
    p_message_rx: ProducerMessageRx,
    p_event_tx: ProducerEventTx<B>,
}

impl<A, B> InnerTask<A, B> {
    fn process_producer_messages(&mut self) {
        loop {
            match self.p_message_rx.poll().unwrap() {
                Async::Ready(message) => {
                    match message {
                        Some(ProducerMessage::Request(n)) => {
                            // just forward the request to the consumer end
                            (&self.c_event_tx).unbounded_send(ConsumerEvent::Request(n));
                        },
                        None => {
                            println!("none received on prod msg");
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
}

impl<A, B> Future for InnerTask<A, B> {
    type Item = ();
    type Error = io::Error;
    
    fn poll(&mut self) -> Poll<(), io::Error> {
        self.process_producer_messages();
        Ok(Async::Ready(()))
    }
}




impl<F, A, B> MapConduit<F, A, B>
    where F: Fn(A) -> B,
          A: Send + 'static,
          B: Send + 'static,
{
    pub fn new(f: F) -> MapConduit<F, A, B> {

        let (c_message_tx, c_message_rx) = mpsc::unbounded::<ConsumerMessage<A>>();
        let (c_event_tx, c_event_rx) = mpsc::unbounded::<ConsumerEvent>();

        let (p_message_tx, p_message_rx) = mpsc::unbounded::<ProducerMessage>();
        let (p_event_tx, p_event_rx) = mpsc::unbounded::<ProducerEvent<B>>();

        let inner = InnerTask {
            c_message_rx,
            c_event_tx,
            p_message_rx,
            p_event_tx,
        };

        tokio::spawn(inner.map_err(|e| {
            eprintln!("{:?}", e);
        }));

        let consumer_half = MapConsumer {
            message_tx: c_message_tx,
            event_rx: Some(c_event_rx),
        };

        let producer_half = MapProducer {
            message_tx: p_message_tx,
            event_rx: Some(p_event_rx),
        };

        MapConduit {
            f,
            in_type: PhantomData,
            out_type: PhantomData,
            consumer: consumer_half,
            producer: producer_half,
        }
    }
}

impl<F, A, B> Consumer<A> for MapConduit<F, A, B>
    where F: Fn(A) -> B,
{
    fn write(&self, data: A) {
        self.consumer.write(data);
    }

    fn end(&self) {
        self.consumer.end();
    }

    fn event_stream(&mut self) -> Option<ConsumerEventRx> {
        self.consumer.event_stream()
    }
}

impl<F, A, B> Producer<B> for MapConduit<F, A, B>
    where F: Fn(A) -> B,
{
    fn request(&mut self, num_items: usize) {
        self.producer.request(num_items);
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<B>> {
        None
    }
}


impl<F, A, B> Conduit<A, B> for MapConduit<F, A, B>
    where F: Fn(A) -> B,
{
    type ConcreteConsumer = MapConsumer<A>;
    type ConcreteProducer = MapProducer<B>;

    fn split(self) -> (MapConsumer<A>, MapProducer<B>) {
        (self.consumer, self.producer)
    }
}


impl<A> Consumer<A> for MapConsumer<A> {
    fn write(&self, data: A) {
        (&self.message_tx).unbounded_send(ConsumerMessage::Write(data)).unwrap();
    }

    fn end(&self) {
        (&self.message_tx).unbounded_send(ConsumerMessage::End).unwrap();
    }

    fn event_stream(&mut self) -> Option<ConsumerEventRx> {
        Option::take(&mut self.event_rx)
    }
}

impl<B> Producer<B> for MapProducer<B> {
    fn request(&mut self, num_items: usize) {
        match (&self.message_tx).unbounded_send(ProducerMessage::Request(num_items)) {
            Ok(_) => {
            },
            Err(_e) => {
                // TODO: properly handle
                //eprintln!("{:?}", e);
            }
        }
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<B>> {
        Option::take(&mut self.event_rx)
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create() {
        //let x = MapConduit {
        //    f: |x: u8| { "hi there" },
        //    in_type: PhantomData,
        //    out_type: PhantomData,
        //};
    }

    #[test]
    fn request_is_forwarded() {
        tokio::run(lazy(|| {
            let mut conduit = MapConduit::new(|x: i32| 0);

            let consumer_events = Consumer::event_stream(&mut conduit).unwrap();

            let task = tokio::spawn(consumer_events.for_each(|event| {
                Ok(())
            })
            .map_err(|_| {}));

            conduit.request(10);

            Ok(())
        }));
    }

    #[test]
    fn split() {

        tokio::run(lazy(|| {
            let mut conduit = MapConduit::new(|x: i32| 0);

            let (mut consumer, mut producer) = conduit.split();

            let consumer_events = consumer.event_stream().unwrap();

            let task = tokio::spawn(consumer_events.for_each(|event| {
                //println!("{:?}", event);
                //assert!(event == ConsumerEvent::Request(12));
                Ok(())
            })
            .map_err(|_| {}));

            producer.request(11);

            Ok(())
        }));
    }
    #[test]
    fn conduit() {
        //let c = MapConduit::new();
    }
}
