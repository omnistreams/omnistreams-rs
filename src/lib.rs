use tokio::io;
//use tokio::net::TcpListener;
use tokio::prelude::*;
use futures::sync::mpsc;


type MessageRx = mpsc::UnboundedReceiver<ConsumerMessage<Vec<u8>>>;
type MessageTx = mpsc::UnboundedSender<ConsumerMessage<Vec<u8>>>;

type EventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type EventTx = mpsc::UnboundedSender<ConsumerEvent>;


pub trait Consumer {
    fn write(&mut self, data: Vec<u8>);
    fn end(&mut self);
    fn next_event(&mut self) -> Option<ConsumerEvent>;
    //fn event_stream(&mut self) -> Stream<Item=ConsumerEvent, Error=io::Error>;
}

#[derive(Debug)]
pub enum ConsumerMessage<T> {
    Write(T),
    End,
}

#[derive(Debug)]
pub enum ConsumerEvent {
    Request(usize),
}

#[derive(Debug)]
pub struct WriteAdapter
{
    message_tx: mpsc::UnboundedSender<ConsumerMessage<Vec<u8>>>,
    //event_rx: EventRx,
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
}

impl<T, U> InnerTask<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    fn new(writer_future: T, message_rx: MessageRx, event_tx: EventTx) -> InnerTask<T, U> {

        (&event_tx).send(ConsumerEvent::Request(1));

        InnerTask {
            state: WriteAdapterState::WaitingForWriter(writer_future),
            message_rx,
            event_tx,
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

        // process any messages from upstream
        loop {
            match self.message_rx.poll().unwrap() {
                Async::Ready(Some(ConsumerMessage::Write(data))) => {
                    println!("write {:?}", data);
                    let tx = &self.event_tx;
                    tx.send(ConsumerEvent::Request(1));
                    task::current().notify();
                },
                Async::Ready(Some(ConsumerMessage::End)) => {
                    println!("end");
                    return Ok(Async::Ready(()));
                },
                Async::NotReady => {
                    break;
                },
                _ => {
                    break;
                }
            }
        }

        Ok(Async::NotReady)
    }
}

impl WriteAdapter
{
    pub fn new<T, U>(writer_future: T) -> (WriteAdapter, EventRx)
        where T: Future<Item=U, Error=io::Error> + Send + 'static,
              U: AsyncWrite + Send + 'static,
    {
        let (message_tx, message_rx) = mpsc::unbounded::<ConsumerMessage<Vec<u8>>>();
        let (event_tx, event_rx) = mpsc::unbounded::<ConsumerEvent>();

        let inner_task = InnerTask::new(writer_future, message_rx, event_tx);
        tokio::spawn(inner_task.map_err(|_| {}));

        (WriteAdapter {
            message_tx,
            //event_rx,
        }, event_rx)
    }

    //pub fn event_stream(&mut self) -> impl Stream<Item=ConsumerEvent, Error=io::Error>
    //{
    //    //futures::stream::ok(ConsumerEvent::Request(1))
    //    self.event_rx
    //}

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

}

//impl<T, U> Future for WriteAdapter<T, U>
//    where T: Future<Item=U, Error=io::Error>,
//          U: AsyncWrite,
//{
//    type Item = ();
//    type Error = io::Error;
//
//    fn poll(&mut self) -> Poll<(), io::Error> {
//
//        Ok(Async::Ready(()))
//
//        //match self.mrx.poll().unwrap() {
//        //    Async::Ready(Some(ConsumerMessage::Write(data))) => {
//        //        println!("write");
//        //        Ok(Async::NotReady)
//        //    },
//        //    Async::Ready(Some(ConsumerMessage::End)) => {
//        //        println!("end");
//        //        Ok(Async::NotReady)
//        //    },
//        //    Async::NotReady => {
//        //        Ok(Async::NotReady)
//        //    },
//        //    _ => {
//        //        Ok(Async::NotReady)
//        //    }
//        //}
//
//        //if self.writer.is_none() {
//        //    match self.writer_future.poll() {
//        //        Ok(Async::Ready(writer)) => {
//        //            self.writer = Some(writer);
//
//        //            task::current().notify();
//        //            Ok(Async::NotReady)
//        //        },
//        //        Ok(Async::NotReady) => {
//        //            Ok(Async::NotReady)
//        //        },
//        //        Err(e) => {
//        //            Err(e)
//        //        },
//        //    }
//        //}
//        //else {
//        //    let writer = self.writer.as_mut().unwrap();
//        //    match writer.poll_write(b"Hi there") {
//        //        Ok(Async::Ready(n)) => {
//        //          println!("wrote: {:?}", n);
//        //          Ok(Async::Ready(()))
//        //        },
//        //        Ok(Async::NotReady) => {
//        //          Ok(Async::NotReady)
//        //        },
//        //        Err(e) => {
//        //            Err(e)
//        //        }
//        //    }
//        //}
//    }
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
