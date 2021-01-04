use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{
    state_machine::{Core, Ended, InGame},
    Connection,
};

use futures::{future::Either, ready, sink::Sink, stream::Stream};
use rsc2_pb::protocol::{self, Status};

pub struct InGameLoop<'sm, 'b> {
    state: Option<InGame<'sm>>,
    framed: Pin<&'b mut Connection>,
}

impl<'sm, 'b> InGameLoop<'sm, 'b> {
    pub fn new(state: InGame<'sm>, framed: Pin<&'b mut Connection>) -> Self {
        Self {
            state: Some(state),
            framed,
        }
    }
}

impl<'sm, 'b> Stream for InGameLoop<'sm, 'b> {
    type Item = Either<<Connection as Stream>::Item, Ended<'sm>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.state.is_none() {
            return Poll::Ready(None);
        }

        let resp = ready!(self.framed.as_mut().poll_next(cx));
        if let Some(Ok(Status::Ended)) = resp.as_ref().map(|r| r.as_ref().map(|r| r.status())) {
            let res = self.state.take().map(|mut state| {
                state.core().replace(Core::Ended {});
                Either::Right(Ended::from(state))
            });
            Poll::Ready(res)
        } else {
            Poll::Ready(resp.map(Either::Left))
        }
    }
}

impl<'sm, 'b> Sink<protocol::Request> for InGameLoop<'sm, 'b> {
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
