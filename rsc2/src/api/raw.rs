//! raw agent api
//!
//! # Introduction
//!
//! The purpose of the raw agent api is to provide mid-level acces to the InGame coordinator
//! meaning you will receive the responses as they come but you won't have access the the producer
//! nor the start and end hooks. Instead proxy methods such as
//! [`on_end`](raw::RawAgent::on_start) and
//! [`on_end`](raw::RawAgent::on_end) will be called.
//!
//! ## On start
//!
//! The first request sent by a [`NewRawAgent`](raw::NewRawAgent) is a
//! [`Response::Observation`](rsc2_pb::api::pb::response::Response::Observation)
//!
//! ## On end
//!
//! [`on_end`](raw::RawAgent::on_end) will be called.
use crate::hook::{AgentHook, NextRequest};
use crate::runtime::Commands;

use rsc2_pb::api as pb;
use std::marker::Unpin;

/// Iterator of request to play exactly one game using the requests fed.
#[allow(missing_debug_implementations)]
pub struct RawRequestGame<A: AgentHook + 'static> {
    messages: Vec<Commands<A>>,
}
impl<A> RawRequestGame<A>
where
    A: AgentHook,
{
    /// Build a new RawRequestGame from a RawAgent
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
impl<A: AgentHook> IntoIterator for RawRequestGame<A> {
    type Item = Commands<A>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

/// Trait for proxy [`AgentHook`](crate::hook::AgentHook) calls
pub trait RawAgent: Send + Unpin {
    /// Called upon receiving a [`Response::GameInfo`](crate::pb::api::response::Response::GameInfo).
    fn on_start(&mut self, response: pb::Response) -> NextRequest;

    /// Called upon receiving anything not handled by on_start and on_end.
    fn on_response(&mut self, response: pb::Response) -> NextRequest;

    /// Called after receiving the first [`Response::GameInfo`](crate::pb::api::response::Response::GameInfo).
    fn on_end(&mut self);
}

/// RawAgent wrapper that passes [`RawAgent`](RawAgent) calls to [`AgentHook`](crate::hook::AgentHook).
#[allow(missing_debug_implementations)]
pub struct NewRawAgent<A: RawAgent>(pub A);

impl<A: RawAgent> AgentHook for NewRawAgent<A> {
    type Producer = std::iter::Cycle<RawProducer>;

    fn on_start_hook(&mut self) -> NextRequest {
        log::trace!("NewRawAgent::on_start_hook");
        NextRequest::Agent(pb::RequestGameInfo::default().into())
    }
    fn on_step_hook(&mut self, response: pb::Response) -> NextRequest {
        log::trace!(
            "NewRawAgent::on_step_hook | id: {}",
            response.id.unwrap_or(std::u32::MAX)
        );
        if let Some(pb::response::Response::GameInfo(_)) = &response.response {
            return self.0.on_start(response);
        }
        self.0.on_response(response)
    }
    fn on_close_hook(&mut self) {
        log::trace!("NewRawAgent::on_close_hook");
        self.0.on_end();
    }
    fn build_producer() -> Self::Producer {
        RawProducer::new(pb::RequestObservation::nofog(0))
            .into_iter()
            .cycle()
    }
}

/// Produces RequestObservations by cloning the initial one.
#[derive(Clone, Debug)]
pub struct RawProducer {
    count: u32,
    request: pb::RequestObservation,
}
impl RawProducer {
    /// Build a new RawProducer
    pub fn new(request: pb::RequestObservation) -> Self {
        Self { count: 0, request }
    }
    fn increment(&mut self) {
        self.count += 1;
    }
}
impl Iterator for RawProducer {
    type Item = pb::request::Request;

    fn next(&mut self) -> Option<Self::Item> {
        let mut request = self.request.clone();
        request.game_loop = Some(self.count);
        self.increment();
        Some(request.into())
    }
}
