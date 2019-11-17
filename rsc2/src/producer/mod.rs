use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::agent::{AgentHook, NextRequest};

use futures::stream::{SplitStream, Stream};
use rsc2_pb::{
    codec::SC2ProtobufCodec,
    sc2_api::{request::Request as rRequest, RequestObservation, Status},
    validate_status,
};
use tokio::{codec::Framed, net::TcpStream};

type SC2SplitStream = SplitStream<Framed<TcpStream, SC2ProtobufCodec>>;

pub(crate) struct StreamAgent<'engine, A, O>
where
    A: AgentHook,
    O: Iterator<Item = rRequest> + Unpin,
{
    agent: &'engine mut A,
    producer: &'engine mut O,
    stream: Pin<&'engine mut SC2SplitStream>,
}

impl<'e, A, O> StreamAgent<'e, A, O>
where
    A: AgentHook,
    O: Iterator<Item = rRequest> + Unpin,
{
    pub(crate) fn new(
        agent: &'e mut A,
        producer: &'e mut O,
        stream: &'e mut SC2SplitStream,
    ) -> Self {
        Self {
            agent,
            producer,
            stream: Pin::new(stream),
        }
    }
}

impl<'e, A, O> Stream for StreamAgent<'e, A, O>
where
    A: AgentHook,
    O: Iterator<Item = rRequest> + Unpin,
{
    type Item = rRequest;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.stream.as_mut().poll_next(cx) {
            Poll::Ready(item) => match item {
                Some(item) => match item {
                    Ok(response) => {
                        log::trace!("|{:?}| response id: {:?}", response.status, response.id);
                        if validate_status!(response.status => Status::Ended).is_ok() {
                            return Poll::Ready(None);
                        }
                        match self.agent.on_step_hook(&response).into() {
                            NextRequest::Agent(request) => Poll::Ready(Some(request)),
                            NextRequest::Observation => Poll::Ready({
                                let item = self.producer.next();
                                debug_assert!(item.is_some());
                                item
                            }),
                        }
                    }
                    Err(err) => {
                        log::error!("{:?}", err);
                        Poll::Pending
                    }
                },
                None => Poll::Ready(None),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct RawProducer {
    count: u32,
}
impl RawProducer {
    pub fn new() -> Self {
        Self { count: 0 }
    }
    pub fn increment(&mut self) {
        self.count += 1;
    }
}
impl Iterator for RawProducer {
    type Item = rRequest;

    fn next(&mut self) -> Option<Self::Item> {
        self.increment();
        Some(RequestObservation::nofog(self.count).into())
    }
}
