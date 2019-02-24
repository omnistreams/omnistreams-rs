use tokio::io;
use tokio::prelude::*;
use futures::sync::mpsc;
use futures::future::lazy;
use super::{
    Producer, ProducerEvent, ProducerEventRx, ProducerEventTx,
    ProducerMessage, ProducerMessageRx, ProducerMessageTx,
};

type Item = i64;

#[derive(Debug)]
pub struct RangeProducer {
    message_tx: ProducerMessageTx,
    event_rx: Option<ProducerEventRx<Item>>,
}

struct InnerTask {
    message_rx: ProducerMessageRx,
    event_tx: ProducerEventTx<Item>,
    demand: usize,
    current_value: Item,
}

impl RangeProducer {
    pub fn new() -> RangeProducer {
        let (message_tx, message_rx) = mpsc::unbounded::<ProducerMessage>();
        let (event_tx, event_rx) = mpsc::unbounded::<ProducerEvent<Item>>();

        let inner_task = InnerTask::new(message_rx, event_tx);
        tokio::spawn(inner_task.map_err(|_| {}));

        RangeProducer {
            message_tx,
            event_rx: Some(event_rx),
        }
    }
}

impl Producer<Item> for RangeProducer {
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

    fn event_stream(&mut self) -> Option<ProducerEventRx<Item>> {
        Option::take(&mut self.event_rx)
    }
}

impl InnerTask
{
    fn new(message_rx: ProducerMessageRx, event_tx: ProducerEventTx<Item>) -> InnerTask {

        InnerTask {
            message_rx,
            event_tx,
            demand: 0,
            current_value: 0,
        }
    }
}

impl Future for InnerTask {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            match self.message_rx.poll() {
                Ok(Async::Ready(Some(ProducerMessage::Request(num_items)))) => {
                    self.demand += num_items;

                    while self.demand > 0 {
                        (&self.event_tx).unbounded_send(ProducerEvent::Data(self.current_value)).unwrap();
                        self.current_value += 1;
                        self.demand -= 1;
                    }
                },
                Ok(Async::Ready(None)) => {
                    return Ok(Async::Ready(()));
                },
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create() {
        tokio::run(lazy(|| {
            RangeProducer::new();

            Ok(())
        }));
    }

    #[test]
    fn simple() {
        tokio::run(lazy(|| {
            let mut producer = RangeProducer::new();

            let events = producer.event_stream().unwrap();

            tokio::spawn(events.for_each(|event| {
                Ok(())
            })
            .map_err(|_| {}));

            producer.request(100);

            Ok(())
        }));
    }
}
