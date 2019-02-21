use tokio::io;
//use tokio::net::TcpListener;
use tokio::prelude::*;
use futures::sync::mpsc;
use futures::try_ready;
use super::{
    Consumer,
    ConsumerMessage,
    ConsumerEvent,
    EventRx,
    EventTx,
    MessageRx,
    MessageTx,
};


#[derive(Debug)]
pub struct WriteAdapter
{
    message_tx: mpsc::UnboundedSender<ConsumerMessage<Vec<u8>>>,
    event_rx: Option<EventRx>,
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
    message_rx: MessageRx,
    event_tx: EventTx,
    demand: usize,
}

impl<T, U> InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    fn new(writer_future: T, message_rx: MessageRx, event_tx: EventTx) -> InnerTask<T, U> {

        let initial_demand = 1;

        (&event_tx).send(ConsumerEvent::Request(initial_demand));

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
                println!("writing");

                // process any messages from upstream
                loop {
                    match self.message_rx.poll().unwrap() {
                        Async::Ready(Some(ConsumerMessage::Write(data))) => {
                            if self.demand == 0 {
                                panic!("WriteAdapter: Attempt to write more than requested");
                            }

                            println!("write {:?}", data);
                            match writer.poll_write(&data) {
                                Ok(Async::Ready(n)) => {
                                    if n != data.len() {
                                        panic!("WriteAdapter: Failed to write all data");
                                    }
                                    (&self.event_tx).send(ConsumerEvent::Request(1));
                                },
                                Ok(Async::NotReady) => {
                                    println!("writer not ready");
                                },
                                Err(e) => {
                                },
                            }
                        },
                        Async::Ready(Some(ConsumerMessage::End)) => {
                            println!("end");
                            return Ok(Async::Ready(()));
                        },
                        Async::NotReady => {
                            break;
                        },
                        _ => {
                            panic!("WriteAdapter: Unknown message");
                            break;
                        }
                    }
                }

                Ok(Async::NotReady)
            },
            WriteAdapterState::WaitingForWriter(ref mut fut) => {
                println!("waiting");
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

impl WriteAdapter
{
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

impl Consumer for WriteAdapter {
    fn write(&mut self, data: Vec<u8>) {
        let tx = &self.message_tx;
        tx.send(ConsumerMessage::Write(data));
    }

    fn end(&mut self) {
        let tx = &self.message_tx;
        tx.send(ConsumerMessage::End);
    }

    fn next_event(&mut self) -> Option<ConsumerEvent> {
        //match self.event_rx.poll().unwrap() {
        //    Async::Ready(Some(event)) => {
        //        Some(event)
        //    },
        //    Async::NotReady => {
        //        None
        //    },
        //    _ => {
        //        None
        //    }
        //}
        None
    }

    fn event_stream(&mut self) -> Option<EventRx> {
        Option::take(&mut self.event_rx)
    }
}
