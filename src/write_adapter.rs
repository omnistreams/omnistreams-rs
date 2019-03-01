use tokio::io;
use tokio::prelude::*;
use futures::sync::mpsc;
use super::{
    Consumer, ConsumerMessage, ConsumerEvent, ConsumerEventRx, ConsumerEventTx, ConsumerMessageRx,
    ConsumerMessageTx
};


#[derive(Debug)]
pub struct WriteAdapter
{
    message_tx: ConsumerMessageTx<Vec<u8>>,
    event_rx: Option<ConsumerEventRx>,
}

#[derive(Debug)]
enum WriteAdapterState<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    WaitingForWriter(T),
    Writing(U),
}

struct InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    state: WriteAdapterState<T, U>,
    message_rx: ConsumerMessageRx<Vec<u8>>,
    event_tx: ConsumerEventTx,
    demand: usize,
}

impl<T, U> InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    fn new(writer_future: T, message_rx: ConsumerMessageRx<Vec<u8>>, event_tx: ConsumerEventTx) -> InnerTask<T, U> {

        let initial_demand = 1;

        println!("send initial");
        (&event_tx).unbounded_send(ConsumerEvent::Request(initial_demand)).unwrap();

        InnerTask {
            state: WriteAdapterState::WaitingForWriter(writer_future),
            message_rx,
            event_tx,
            demand: initial_demand,
        }
    }
}

impl<T, U> Future for InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        match self.state {
            WriteAdapterState::Writing(ref mut writer) => {

                // process any messages from upstream
                loop {
                    match self.message_rx.poll().unwrap() {
                        Async::Ready(Some(ConsumerMessage::Write(data))) => {
                            if self.demand == 0 {
                                panic!("WriteAdapter: Attempt to write more than requested");
                            }

                            match writer.poll_write(&data) {
                                Ok(Async::Ready(n)) => {
                                    if n != data.len() {
                                        panic!("WriteAdapter: Failed to write all data");
                                    }
                                    (&self.event_tx).unbounded_send(ConsumerEvent::Request(1)).unwrap();
                                },
                                Ok(Async::NotReady) => {
                                },
                                Err(_e) => {
                                },
                            }
                        },
                        Async::Ready(Some(ConsumerMessage::End)) => {
                            return Ok(Async::Ready(()));
                        },
                        Async::Ready(None) => {
                            return Ok(Async::Ready(()));
                        },
                        Async::NotReady => {
                            break;
                        },
                    }
                }

                Ok(Async::NotReady)
            },
            WriteAdapterState::WaitingForWriter(ref mut fut) => {
                return match fut.poll() {
                    Ok(Async::Ready(writer)) => {
                        self.state = WriteAdapterState::Writing(writer);
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
    }
}

impl WriteAdapter {
    pub fn new<T, U>(writer_future: T) -> WriteAdapter
        where T: Future<Item=U, Error=io::Error> + Send + 'static,
              U: AsyncWrite + Send + 'static,
    {
        let (message_tx, message_rx) = mpsc::unbounded::<ConsumerMessage<Vec<u8>>>();
        let (event_tx, event_rx) = mpsc::unbounded::<ConsumerEvent>();

        let inner_task = InnerTask::new(writer_future, message_rx, event_tx);
        tokio::spawn(inner_task.map_err(|_| {}));

        WriteAdapter {
            message_tx,
            event_rx: Some(event_rx),
        }
    }
}

impl Consumer<Vec<u8>> for WriteAdapter {
    fn write(&self, data: Vec<u8>) {
        let tx = &self.message_tx;
        tx.unbounded_send(ConsumerMessage::Write(data)).unwrap();
    }

    fn end(&self) {
        let tx = &self.message_tx;
        tx.unbounded_send(ConsumerMessage::End).unwrap();
    }

    fn event_stream(&mut self) -> Option<ConsumerEventRx> {
        Option::take(&mut self.event_rx)
    }
}
