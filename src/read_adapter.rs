use tokio::io;
use tokio::prelude::*;
use futures::sync::mpsc;
use super::{Producer, ProducerEvent, ProducerEventRx};

type Item = Vec<u8>;

pub struct ReadAdapter {
    event_rx: Option<ProducerEventRx<Item>>,
}

impl ReadAdapter {
    pub fn new<T, U>(reader_future: T) -> ReadAdapter
        where T: Future<Item=U, Error=io::Error> + Send + 'static,
              U: AsyncRead + Send + 'static,
    {
        let (event_tx, event_rx) = mpsc::unbounded::<ProducerEvent<Item>>();

        ReadAdapter {
            event_rx: Some(event_rx),
        }
    }
}

impl Producer<Item> for ReadAdapter {
    fn request(&mut self, num_items: usize) {
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<Item>> {
        Option::take(&mut self.event_rx)
    }
}
