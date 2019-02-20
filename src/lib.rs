use tokio::io;
//use tokio::net::TcpListener;
use tokio::prelude::*;
use futures::sync::mpsc;


type MessageRx = mpsc::UnboundedReceiver<ConsumerMessage>;
type MessageTx = mpsc::UnboundedSender<ConsumerMessage>;

pub trait Consumer {
    fn message_tx(&self) -> MessageTx;
}

#[derive(Debug)]
pub enum ConsumerMessage {
    Write(Vec<u8>),
    End,
}

pub enum ConsumerEvent {
}

#[derive(Debug)]
pub struct WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    writer_future: T,
    writer: Option<U>,
    mrx: MessageRx,
    message_tx: MessageTx,
}

impl<T, U> WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    pub fn new(writer_future: T) -> WriteAdapter<T, U> {
        
        let (message_tx, mrx) = mpsc::unbounded::<ConsumerMessage>();

        WriteAdapter {
            writer_future,
            writer: None,
            mrx,
            message_tx,
        }
    }
}

impl<T, U> Consumer for WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    fn message_tx(&self) -> MessageTx {
        self.message_tx.clone()
    }
}

impl<T, U> Future for WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        match self.mrx.poll().unwrap() {
            Async::Ready(Some(ConsumerMessage::Write(data))) => {
                println!("write");
                Ok(Async::NotReady)
            },
            Async::Ready(Some(ConsumerMessage::End)) => {
                println!("end");
                Ok(Async::NotReady)
            },
            Async::NotReady => {
                Ok(Async::NotReady)
            },
            _ => {
                Ok(Async::NotReady)
            }
        }

        //if self.writer.is_none() {
        //    match self.writer_future.poll() {
        //        Ok(Async::Ready(writer)) => {
        //            self.writer = Some(writer);

        //            task::current().notify();
        //            Ok(Async::NotReady)
        //        },
        //        Ok(Async::NotReady) => {
        //            Ok(Async::NotReady)
        //        },
        //        Err(e) => {
        //            Err(e)
        //        },
        //    }
        //}
        //else {
        //    let writer = self.writer.as_mut().unwrap();
        //    match writer.poll_write(b"Hi there") {
        //        Ok(Async::Ready(n)) => {
        //          println!("wrote: {:?}", n);
        //          Ok(Async::Ready(()))
        //        },
        //        Ok(Async::NotReady) => {
        //          Ok(Async::NotReady)
        //        },
        //        Err(e) => {
        //            Err(e)
        //        }
        //    }
        //}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
