use futures::sync::mpsc;

mod read_adapter;
mod write_adapter;

pub use self::read_adapter::ReadAdapter;
pub use self::write_adapter::WriteAdapter;


type MessageRx<T> = mpsc::UnboundedReceiver<ConsumerMessage<T>>;
type MessageTx<T> = mpsc::UnboundedSender<ConsumerMessage<T>>;

type EventRx = mpsc::UnboundedReceiver<ConsumerEvent>;
type EventTx = mpsc::UnboundedSender<ConsumerEvent>;

type ProducerEventRx<T> = mpsc::UnboundedReceiver<ProducerEvent<T>>;


pub trait Consumer<T> {
    fn write(&mut self, data: T);
    fn end(&mut self);
    fn event_stream(&mut self) -> Option<EventRx>;
}

pub trait Producer<T> {
    fn request(&mut self, num_items: usize);
    fn event_stream(&mut self) -> Option<ProducerEventRx<T>>;
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

pub enum ProducerEvent<T> {
    Data(T),
    End,
}
