use tokio::io;
use tokio::prelude::*;
use futures::sync::mpsc;
use super::{
    Producer, ProducerEvent, ProducerEventRx, ProducerEventTx,
    ProducerMessage, ProducerMessageRx, ProducerMessageTx,
};

type Item = Vec<u8>;

const CHUNK_SIZE: usize = 1024;

#[derive(Debug)]
pub struct ReadAdapter {
    message_tx: ProducerMessageTx,
    event_rx: Option<ProducerEventRx<Item>>,
}

enum ReadAdapterState<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncRead,
{
    WaitingForReader(T),
    Reading(U),
}

struct InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncRead,
{
    state: ReadAdapterState<T, U>,
    message_rx: ProducerMessageRx,
    event_tx: ProducerEventTx<Item>,
    demand: usize,
}

impl ReadAdapter {
    pub fn new<T, U>(reader_future: T) -> ReadAdapter
        where T: Future<Item=U, Error=io::Error> + Send + 'static,
              U: AsyncRead + Send + 'static,
    {
        let (message_tx, message_rx) = mpsc::unbounded::<ProducerMessage>();
        let (event_tx, event_rx) = mpsc::unbounded::<ProducerEvent<Item>>();

        let inner_task = InnerTask::new(reader_future, message_rx, event_tx);
        tokio::spawn(inner_task.map_err(|_| {}));

        ReadAdapter {
            message_tx,
            event_rx: Some(event_rx),
        }
    }
}

impl Producer<Item> for ReadAdapter {
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

impl<T, U> InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncRead,
{
    fn new(reader_future: T, message_rx: ProducerMessageRx, event_tx: ProducerEventTx<Item>) -> InnerTask<T, U> {

        InnerTask {
            state: ReadAdapterState::WaitingForReader(reader_future),
            message_rx,
            event_tx,
            demand: 0,
        }
    }
}

impl<T, U> Future for InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncRead,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        match self.state {
            ReadAdapterState::Reading(ref mut reader) => {

                loop {
                    match self.message_rx.poll().unwrap() {
                        Async::Ready(Some(ProducerMessage::Request(num_items))) => {
                            self.demand += num_items;
                        },
                        Async::Ready(None) => {
                            return Ok(Async::Ready(()));
                        },
                        Async::NotReady => {
                            break;
                        },
                    }
                }

                while self.demand > 0 {
                    let mut buf = [0; CHUNK_SIZE];

                    match reader.poll_read(&mut buf) {
                        Ok(Async::Ready(n)) => {

                            self.demand -= 1;

                            (&self.event_tx).unbounded_send(ProducerEvent::Data(buf[0..n].to_vec())).unwrap();

                            if n != CHUNK_SIZE {
                                //eprintln!("wrong chunk size: {}", n);
                            }

                            if n == 0 {
                                return Ok(Async::Ready(()));
                            }
                        },
                        Ok(Async::NotReady) => {
                            return Ok(Async::NotReady);
                        },
                        Err(_e) => {
                            break;
                        },
                    };
                }
            },
            ReadAdapterState::WaitingForReader(ref mut fut) => {
                return match fut.poll() {
                    Ok(Async::Ready(reader)) => {
                        self.state = ReadAdapterState::Reading(reader);
                        task::current().notify();
                        Ok(Async::NotReady)
                    },
                    Ok(Async::NotReady) => {
                        Ok(Async::NotReady)
                    },
                    Err(e) => {
                        Err(e)
                    }
                }
            },
        }

        //(&self.event_tx).send(ProducerEvent::Data(vec![66,66,66]));
        //Ok(Async::Ready(()))
        Ok(Async::NotReady)
    }
}
