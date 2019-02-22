use futures::sync::mpsc;

mod write_adapter;

pub use self::write_adapter::WriteAdapter;


type MessageRx<T> = mpsc::UnboundedReceiver<ConsumerMessage<T>>;
type MessageTx<T> = mpsc::UnboundedSender<ConsumerMessage<T>>;

type EventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type EventTx = mpsc::UnboundedSender<ConsumerEvent>;


pub trait Consumer<T> {
    fn write(&mut self, data: T);
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
