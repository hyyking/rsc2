use crate::hook::{AgentHook, NextRequest};
use crate::runtime::Commands;

use rsc2_pb::api as pb;
use std::marker::Unpin;

pub struct RawRequestGame<A>
where
    A: AgentHook + 'static,
{
    messages: Vec<Commands<A>>,
}
impl<A> RawRequestGame<A>
where
    A: AgentHook,
{
    pub fn new(agent: A, create: pb::RequestCreateGame, join: pb::RequestJoinGame) -> Self {
        let mut messages = Vec::with_capacity(5);
        messages.push(Commands::Launched {
            socket: "127.0.0.1:5000".parse().unwrap(),
        });
        messages.push(Commands::CreateGame { request: create });
        messages.push(Commands::JoinGame {
            request: join,
            agent,
        });
        messages.push(Commands::LeaveGame);
        messages.push(Commands::QuitGame);
        Self { messages }
    }
}
impl<A> IntoIterator for RawRequestGame<A>
where
    A: AgentHook,
{
    type Item = Commands<A>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

pub trait RawAgent: Send + Unpin {
    fn on_response(&mut self, response: pb::Response) -> NextRequest;
}

pub struct NewRawAgent<A: RawAgent>(pub A);
impl<A> AgentHook for NewRawAgent<A>
where
    A: RawAgent,
{
    type Producer = std::iter::Cycle<RawProducer>;

    fn on_start_hook(&mut self) -> NextRequest {
        log::trace!("RawAgent::on_start_hook");
        NextRequest::Agent(pb::RequestGameInfo::default().into())
    }
    fn on_step_hook(&mut self, response: pb::Response) -> NextRequest {
        log::trace!(
            "RawAgent::on_step_hook | id: {}",
            response.id.unwrap_or(std::u32::MAX)
        );
        self.0.on_response(response)
    }
    fn on_close_hook(&mut self) {
        log::trace!("RawAgent::on_close_hook");
    }
    fn build_producer() -> Self::Producer {
        RawProducer::new().into_iter().cycle()
    }
}

#[derive(Clone)]
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
    type Item = pb::request::Request;

    fn next(&mut self) -> Option<Self::Item> {
        self.increment();
        Some(pb::RequestObservation::nofog(self.count).into())
    }
}
