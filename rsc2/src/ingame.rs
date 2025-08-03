use std::convert::identity;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{
    Connection,
    state_machine::{Core, Ended, InGame},
};
use either::Either;

use futures::{ready, sink::Sink, stream::Stream};
use rsc2_pb::protocol::{self, Status};

pub struct InGameLoop<'sm, 'b> {
    state: Either<Option<InGame<'sm>>, Ended<'sm>>,
    framed: Pin<&'b mut Connection>,
}

impl<'sm, 'b> InGameLoop<'sm, 'b> {
    pub fn new(state: InGame<'sm>, framed: Pin<&'b mut Connection>) -> Self {
        Self {
            state: Either::Left(Some(state)),
            framed,
        }
    }
}

impl<'sm, 'b> Stream for InGameLoop<'sm, 'b> {
    type Item = <Connection as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.state.is_right() {
            return Poll::Ready(None);
        }

        let response: Option<Result<protocol::Response, std::io::Error>> =
            ready!(self.framed.as_mut().poll_next(cx));

        let match_ended = response.as_ref().is_some_and(|ok| {
            matches!(
                ok.as_ref().map(protocol::Response::status),
                Ok(Status::Ended)
            )
        });
        if match_ended {
            let state = std::mem::replace(&mut self.state, Either::Left(None));

            let ended = state.either(
                |ingame| {
                    let mut ingame = ingame.expect("In game state was expected");
                    ingame.core().replace(Core::Ended {});
                    Ended::from(ingame)
                },
                identity,
            );
            self.state = Either::Right(ended);

            return Poll::Ready(None);
        }

        Poll::Ready(response)
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
