use rsc2_pb::sc2_api::{request::Request as rRequest, Response};

pub enum NextRequest {
    Agent(rRequest),
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

pub trait AgentHook: Send + Unpin {
    fn on_start_hook(&mut self) -> NextRequest;
    fn on_step_hook(&mut self, response: &Response) -> NextRequest;
    fn on_close_hook(&mut self);
}
