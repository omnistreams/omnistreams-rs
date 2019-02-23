use super::{
    Consumer, ConsumerEventRx, Producer, ProducerEventRx, Conduit,
    ConsumerMessage, ConsumerEvent, ProducerMessage, ProducerEvent,
    ConsumerMessageTx, ProducerMessageTx 
};
use std::marker::PhantomData;
use futures::sync::mpsc;


pub struct MapConduit<F, A, B>
    where F: Fn(A) -> B
{
    f: F,
    in_type: PhantomData<A>,
    out_type: PhantomData<B>,
    c_message_tx: ConsumerMessageTx<A>,
    c_event_rx: Option<ConsumerEventRx>,
    p_message_tx: ProducerMessageTx,
    p_event_rx: Option<ProducerEventRx<B>>,
}

impl<F, A, B> MapConduit<F, A, B>
    where F: Fn(A) -> B,
{
    pub fn new(f: F) -> MapConduit<F, A, B> {

        let (c_message_tx, c_message_rx) = mpsc::unbounded::<ConsumerMessage<A>>();
        let (c_event_tx, c_event_rx) = mpsc::unbounded::<ConsumerEvent>();

        let (p_message_tx, p_message_rx) = mpsc::unbounded::<ProducerMessage>();
        let (p_event_tx, p_event_rx) = mpsc::unbounded::<ProducerEvent<B>>();

        MapConduit {
            f,
            in_type: PhantomData,
            out_type: PhantomData,
            c_message_tx: c_message_tx,
            c_event_rx: Some(c_event_rx),
            p_message_tx: p_message_tx,
            p_event_rx: Some(p_event_rx),
        }
    }
}

impl<F, A, B> Consumer<A> for MapConduit<F, A, B>
    where F: Fn(A) -> B,
{
    fn write(&mut self, data: A) {
    }

    fn end(&mut self) {
    }

    fn event_stream(&mut self) -> Option<ConsumerEventRx> {
        None
    }
}

impl<F, A, B> Producer<B> for MapConduit<F, A, B>
    where F: Fn(A) -> B,
{
    fn request(&mut self, num_items: usize) {
    }

    fn event_stream(&mut self) -> Option<ProducerEventRx<B>> {
        None
    }
}


impl<F, A, B> Conduit<A, B> for MapConduit<F, A, B>
    where F: Fn(A) -> B,
{
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create() {
        //let x = MapConduit {
        //    f: |x: u8| { "hi there" },
        //    in_type: PhantomData,
        //    out_type: PhantomData,
        //};
    }

    #[test]
    fn new() {
        MapConduit::new(|x: i32| {
            0
        });
    }

    #[test]
    fn conduit() {
        //let c = MapConduit::new();
    }
}
