use std::fmt::Debug;
use tokio::prelude::*;
use futures::sync::mpsc;
use super::{
    Consumer, ConsumerMessage, ConsumerEvent, ConsumerEventRx, ConsumerEventTx, ConsumerMessageRx,
    ConsumerMessageTx, CancelReason
};

type Message = Vec<u8>;


#[derive(Debug)]
pub struct SinkAdapter {
    message_tx: ConsumerMessageTx<Message>,
    event_rx: Option<ConsumerEventRx>,
}

//#[derive(Debug)]
//enum SinkAdapterState<S>
//    where S: Sink<SinkItem=Message, SinkError=io::Error> + Send + 'static,
//{
//    WaitingForWriter(S),
//    Writing(S),
//}

struct InnerTask<S, E>
    where S: Sink<SinkItem=Message, SinkError=E> + Send + 'static,
          E: 'static + Debug,
{
    sink: S,
    message_rx: ConsumerMessageRx<Message>,
    event_tx: ConsumerEventTx,
    demand: usize,
    buffered: Option<Message>,
}

impl<S, E> InnerTask<S, E>
    where S: Sink<SinkItem=Message, SinkError=E> + Send + 'static,
          E: 'static + Debug,
{
    fn new(sink: S, message_rx: ConsumerMessageRx<Message>, event_tx: ConsumerEventTx) -> Self {

        let initial_demand = 1;

        (&event_tx).unbounded_send(ConsumerEvent::Request(initial_demand)).unwrap();

        Self {
            sink,
            message_rx,
            event_tx,
            demand: initial_demand,
            buffered: None,
        }
    }

    fn send_or_buffer(&mut self, data: Message) {
        match self.sink.start_send(data) {
            Ok(AsyncSink::Ready) => {
                (&self.event_tx).unbounded_send(ConsumerEvent::Request(1)).unwrap();
            },
            Ok(AsyncSink::NotReady(data)) => {
                self.buffered = Some(data);
            },
            Err(_e) => {
                (&self.event_tx).unbounded_send(ConsumerEvent::Cancellation(CancelReason::Disconnected)).unwrap();
            },
        }
    }
}

impl<S, E> Future for InnerTask<S, E>
    where S: Sink<SinkItem=Message, SinkError=E> + Send + 'static,
          E: 'static + Debug,
{
    type Item = ();
    type Error = E;

    fn poll(&mut self) -> Poll<(), E> {

        let buffered = Option::take(&mut self.buffered);

        match buffered {
            Some(data) => {
                match self.sink.poll_complete() {
                    Ok(Async::Ready(())) => {
                        self.send_or_buffer(data);
                    },
                    Ok(Async::NotReady) => {
                    },
                    Err(e) => {
                        eprintln!("SinkAdapter poll err: {:?}", e);
                    },
                }
            },
            None => {
            },
        }

        // Only process messages if there isn't already something in the
        // buffer. Otherwise there's nowhere to put them.
        while self.buffered.is_none() {
            match self.message_rx.poll().unwrap() {
                Async::Ready(Some(ConsumerMessage::Write(data))) => {
                    if self.demand == 0 {
                        panic!("SinkAdapter: Attempt to write more than requested");
                    }

                    self.send_or_buffer(data);
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
    }
}

impl SinkAdapter {
    pub fn new<S, E>(sink: S) -> Self
        where S: Sink<SinkItem=Message, SinkError=E> + Send + 'static,
              E: 'static + Debug,
    {
        let (message_tx, message_rx) = mpsc::unbounded::<ConsumerMessage<Message>>();
        let (event_tx, event_rx) = mpsc::unbounded::<ConsumerEvent>();

        let inner_task = InnerTask::new(sink, message_rx, event_tx);
        tokio::spawn(inner_task.map_err(|_| {}));

        Self {
            message_tx,
            event_rx: Some(event_rx),
        }
    }
}

impl Consumer<Message> for SinkAdapter {
    fn write(&self, data: Message) {
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
