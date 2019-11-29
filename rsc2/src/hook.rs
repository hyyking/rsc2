//! hooks called by the coordinator on events.
use rsc2_pb::api::{request::Request as rRequest, Response};

/// Next request for a gameloop.
#[derive(Debug, Clone)]
pub enum NextRequest {
    /// Agent will be producing the next request.
    Agent(rRequest),

    /// Producer will be producing the next request.
    Observation,
}

impl From<Option<rRequest>> for NextRequest {
    fn from(other: Option<rRequest>) -> Self {
        match other {
            Some(req) => NextRequest::Agent(req),
            None => NextRequest::Observation,
        }
    }
}

/// Base trait for hooking an agent to a coordinator that is InGame.
pub trait AgentHook: Send + Unpin {
    /// A producer is an infinite iterator that produces [`Request`](crate::pb::api::request::Request)s
    type Producer: Iterator<Item = rRequest> + Send;

    /// First function called by the coordinator. It should always produce a request since it will
    /// allow the coordinator to spawn the response hooks.
    fn on_start_hook(&mut self) -> NextRequest;

    /// Called on every responses
    fn on_step_hook(&mut self, response: Response) -> NextRequest;

    /// Called after the end of a game, should be use as a way to reset the agent's inner state.
    fn on_close_hook(&mut self);

    /// Builds a producer before the start of the game.
    fn build_producer() -> Self::Producer;
}
