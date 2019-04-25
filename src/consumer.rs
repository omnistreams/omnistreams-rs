use futures::sync::mpsc;
use futures::{Stream};
use super::CancelReason;


pub type ConsumerMessageRx<T> = mpsc::UnboundedReceiver<ConsumerMessage<T>>;
pub type ConsumerMessageTx<T> = mpsc::UnboundedSender<ConsumerMessage<T>>;

pub type ConsumerEventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
pub type ConsumerEventTx = mpsc::UnboundedSender<ConsumerEvent>;


#[derive(Debug)]
pub enum ConsumerMessage<T> {
    Write(T),
    End,
}

#[derive(PartialEq, Clone, Debug)]
pub enum ConsumerEvent {
    Request(usize),
    Cancellation(CancelReason),
}
pub trait Consumer<T> {
    fn write(&self, data: T);
    fn end(&self);
    fn event_stream(&mut self) -> Option<ConsumerEventRx>;
    fn set_event_stream(&mut self, event_stream: ConsumerEventRx);
}


