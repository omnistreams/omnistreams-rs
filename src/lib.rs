use futures::sync::mpsc;

mod write_adapter;

pub use self::write_adapter::WriteAdapter;


type MessageRx = mpsc::UnboundedReceiver<ConsumerMessage<Vec<u8>>>;
type MessageTx = mpsc::UnboundedSender<ConsumerMessage<Vec<u8>>>;

type EventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type EventTx = mpsc::UnboundedSender<ConsumerEvent>;


pub trait Consumer {
    fn write(&mut self, data: Vec<u8>);
    fn end(&mut self);
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
