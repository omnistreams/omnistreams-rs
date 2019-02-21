use tokio::io;
//use tokio::net::TcpListener;
use tokio::prelude::*;
use futures::sync::mpsc;
use futures::try_ready;

mod write_adapter;

pub use self::write_adapter::WriteAdapter;


type MessageRx = mpsc::UnboundedReceiver<ConsumerMessage<Vec<u8>>>;
type MessageTx = mpsc::UnboundedSender<ConsumerMessage<Vec<u8>>>;

type EventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type EventTx = mpsc::UnboundedSender<ConsumerEvent>;


pub trait Consumer {
    fn write(&mut self, data: Vec<u8>);
    fn end(&mut self);
    fn next_event(&mut self) -> Option<ConsumerEvent>;
    fn event_stream(&mut self) -> Option<EventRx>;
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
