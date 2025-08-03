use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{
    Connection,
    state_machine::{Core, Ended, InGame},
};
use either::Either;

use futures::io;
use futures::{ready, sink::Sink, stream::Stream};
use rsc2_pb::protocol::{self, Status};

pub struct InGameListener<'sm, 'b> {
    state: Either<Option<InGame<'sm>>, Ended<'sm>>,
    framed: Pin<&'b mut Connection>,
}

fn try_end_game<'sm>(
    state: Either<Option<InGame<'sm>>, Ended<'sm>>,
) -> Result<Ended<'sm>, io::Error> {
    state.either(
        |ingame| {
            if let Some(mut ingame) = ingame {
                ingame.core().replace(Core::Ended {});
                Ok(Ended::from(ingame))
            } else {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Game is already ended",
                ))
            }
        },
        Ok,
    )
}

impl<'sm, 'b> InGameListener<'sm, 'b> {
    pub fn new(state: InGame<'sm>, framed: Pin<&'b mut Connection>) -> Self {
        Self {
            state: Either::Left(Some(state)),
            framed,
        }
    }
    pub fn into_ended(self) -> Ended<'sm> {
        try_end_game(self.state).unwrap()
    }
}

impl<'sm, 'b> Stream for InGameListener<'sm, 'b> {
    type Item = <Connection as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.state.is_right() {
            return Poll::Ready(None);
        }

        let response = ready!(self.framed.as_mut().poll_next(cx));

        let match_ended = response.as_ref().is_some_and(|ok| {
            matches!(
                ok.as_ref().map(protocol::Response::status),
                Ok(Status::Ended)
            )
        });
        if match_ended {
            let state = std::mem::replace(&mut self.state, Either::Left(None));
            self.state = Either::Right(try_end_game(state).unwrap());

            return Poll::Ready(None);
        }

        Poll::Ready(response)
    }
}

impl<'sm, 'b> Sink<protocol::Request> for InGameListener<'sm, 'b> {
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
