use tokio::io;
//use tokio::net::TcpListener;
use tokio::prelude::*;

pub trait Consumer {
}

pub struct WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    writer_future: T,
    writer: Option<U>,
}

impl<T, U> WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    pub fn new(writer_future: T) -> WriteAdapter<T, U> {
        WriteAdapter {
            writer_future,
            writer: None,
        }
    }
}

impl<T, U> Consumer for WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
}

impl<T, U> Future for WriteAdapter<T, U>
    where T: Future<Item=U, Error=io::Error>,
          U: AsyncWrite,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        if self.writer.is_none() {
            match self.writer_future.poll() {
                Ok(Async::Ready(writer)) => {
                    self.writer = Some(writer);

                    task::current().notify();
                    Ok(Async::NotReady)
                },
                Ok(Async::NotReady) => {
                    Ok(Async::NotReady)
                },
                Err(e) => {
                    Err(e)
                },
            }
        }
        else {
            let writer = self.writer.as_mut().unwrap();
            match writer.poll_write(b"Hi there") {
                Ok(Async::Ready(n)) => {
                  println!("wrote: {:?}", n);
                  Ok(Async::Ready(()))
                },
                Ok(Async::NotReady) => {
                  Ok(Async::NotReady)
                },
                Err(e) => {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
