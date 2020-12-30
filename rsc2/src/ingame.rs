use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{
    state_machine::{Ended, Engine, InGame},
    Connection,
};

use futures::{future::Either, ready, sink::Sink, stream::Stream};
use rsc2_pb::protocol::{self, Status};

pub struct InGameLoop<'a> {
    state: Option<Engine<InGame>>,
    framed: Pin<&'a mut Connection>,
}

impl<'a> InGameLoop<'a> {
    pub fn new(state: Engine<InGame>, framed: Pin<&'a mut Connection>) -> Self {
        Self {
            state: Some(state),
            framed,
        }
    }
}

impl<'a> Stream for InGameLoop<'a> {
    type Item = Either<<Connection as Stream>::Item, Engine<Ended>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.state.is_none() {
            return Poll::Ready(None);
        }

        let resp = ready!(self.framed.as_mut().poll_next(cx));
        if let Some(Ok(Status::Ended)) = resp.as_ref().map(|r| r.as_ref().map(|r| r.status())) {
            let res = self
                .state
                .take()
                .map(|state| Either::Right(state.as_ended()));
            Poll::Ready(res)
        } else {
            Poll::Ready(resp.map(Either::Left))
        }
    }
}

impl<'a> Sink<protocol::Request> for InGameLoop<'a> {
    type Error = <Connection as Sink<protocol::Request>>::Error;
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<protocol::Request>::poll_ready(self.framed.as_mut(), cx)
    }
    fn start_send(mut self: Pin<&mut Self>, item: protocol::Request) -> Result<(), Self::Error> {
        Sink::<protocol::Request>::start_send(self.framed.as_mut(), item)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<protocol::Request>::poll_flush(self.framed.as_mut(), cx)
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<protocol::Request>::poll_close(self.framed.as_mut(), cx)
    }
}
