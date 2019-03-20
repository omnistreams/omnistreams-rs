use super::{
    Consumer, ConsumerEventRx, Producer, ProducerEventRx, Conduit,
    ConsumerMessage, ConsumerEvent, ProducerMessage, ProducerEvent,
    ConsumerMessageTx, ProducerMessageTx,
    ConsumerMessageRx, ConsumerEventTx, ProducerMessageRx, ProducerEventTx,
    Streamer, CancelReason,
};
use std::marker::PhantomData;
use futures::sync::mpsc;
use tokio::io;
use tokio::prelude::*;


#[derive(Debug)]
pub struct MapConduit<A, B> {
    in_type: PhantomData<A>,
    out_type: PhantomData<B>,
    consumer: MapConsumer<A>,
    producer: MapProducer<B>,
}

#[derive(Debug)]
pub struct MapConsumer<A> {
    message_tx: ConsumerMessageTx<A>,
    event_rx: Option<ConsumerEventRx>,
}

#[derive(Debug)]
pub struct MapProducer<B> {
    message_tx: ProducerMessageTx,
    event_rx: Option<ProducerEventRx<B>>,
}

struct InnerTask<F, A, B>
    where F: FnMut(A) -> B + Send
{
    f: F,
    c_message_rx: ConsumerMessageRx<A>,
    c_event_tx: ConsumerEventTx,
    p_message_rx: ProducerMessageRx,
    p_event_tx: ProducerEventTx<B>,
    ended: bool,
}

impl<F, A, B> InnerTask<F, A, B>
    where F: FnMut(A) -> B + Send
{
    fn process_producer_messages(&mut self) {
        loop {
            match self.p_message_rx.poll().unwrap() {
                Async::Ready(message) => {
                    match message {
                        Some(ProducerMessage::Request(n)) => {
                            // just forward the request to the consumer end
                            (&self.c_event_tx).unbounded_send(ConsumerEvent::Request(n)).unwrap();
                        },
                        Some(ProducerMessage::Cancel(_reason)) => {
                            // TODO: implement
                            panic!("Cancel MapConduit");
                        },
                        None => {
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

    fn process_consumer_messages(&mut self) {
        loop {
            match self.c_message_rx.poll().unwrap() {
                Async::Ready(message) => {
                    match message {
                        Some(ConsumerMessage::Write(data)) => {
                            let mapped = (self.f)(data);
                            (&self.p_event_tx).unbounded_send(ProducerEvent::Data(mapped)).unwrap();
                        },
                        Some(ConsumerMessage::End) => {
                            (&self.p_event_tx).unbounded_send(ProducerEvent::End).unwrap();
                            self.ended = true;
                            break;
                        },
                        None => {
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

impl<F, A, B> Future for InnerTask<F, A, B>
    where F: FnMut(A) -> B + Send
{
    type Item = ();
    type Error = io::Error;
    
    fn poll(&mut self) -> Poll<(), io::Error> {
        self.process_producer_messages();
        self.process_consumer_messages();
        //Ok(Async::Ready(()))

        if self.ended {
            Ok(Async::Ready(()))
        }
        else {
            //task::current().notify();
            Ok(Async::NotReady)
        }
    }
}




impl<A, B> MapConduit<A, B>
    where A: Send + 'static,
          B: Send + 'static,
{
    pub fn new<F: FnMut(A) -> B + Send + 'static>(f: F) -> MapConduit<A, B> {

        let (c_message_tx, c_message_rx) = mpsc::unbounded::<ConsumerMessage<A>>();
        let (c_event_tx, c_event_rx) = mpsc::unbounded::<ConsumerEvent>();

        let (p_message_tx, p_message_rx) = mpsc::unbounded::<ProducerMessage>();
        let (p_event_tx, p_event_rx) = mpsc::unbounded::<ProducerEvent<B>>();

        let inner = InnerTask {
            f,
            c_message_rx,
            c_event_tx,
            p_message_rx,
            p_event_tx,
            ended: false,
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
            in_type: PhantomData,
            out_type: PhantomData,
            consumer: consumer_half,
            producer: producer_half,
        }
    }
}

impl<A, B> Consumer<A> for MapConduit<A, B> {
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

impl<A, B> Streamer for MapConduit<A, B> {
    fn cancel(&mut self, _reason: CancelReason) {
        // TODO: implement cancel
    }
}

impl<A, B> Producer<B> for MapConduit<A, B> {
    fn request(&mut self, num_items: usize) {
        self.producer.request(num_items);
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<B>> {
        self.producer.event_stream()
    }
}


impl<A, B> Conduit<A, B> for MapConduit<A, B> {
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

impl<B> Streamer for MapProducer<B> {
    fn cancel(&mut self, _reason: CancelReason) {
        // TODO: implement cancel
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

    use futures::future::lazy;
    use super::*;

    #[test]
    fn request_is_forwarded() {
        tokio::run(lazy(|| {
            let mut conduit = MapConduit::new(|_: i32| 0);

            let consumer_events = Consumer::event_stream(&mut conduit).unwrap();

            tokio::spawn(consumer_events.for_each(|_event| {
                Ok(())
            })
            .map_err(|_| {}));

            conduit.request(10);
            conduit.end();

            Ok(())
        }));
    }

    #[test]
    fn split() {

        tokio::run(lazy(|| {
            let conduit = MapConduit::new(|_: i32| 0);

            let (mut consumer, mut producer) = conduit.split();

            let consumer_events = consumer.event_stream().unwrap();

            tokio::spawn(consumer_events.for_each(|_event| {
                //println!("{:?}", event);
                //assert!(event == ConsumerEvent::Request(12));
                Ok(())
            })
            .map_err(|_| {}));

            producer.request(11);
            consumer.end();

            Ok(())
        }));
    }
}
