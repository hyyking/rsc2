use rsc2_pb::api::{request::Request as rRequest, Response};

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

impl std::fmt::Debug for NextRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct(&format!(
            "NextRequest::{}",
            match self {
                Self::Agent(_) => "Agent",
                Self::Observation => "Observation",
            }
        ))
        .finish()
    }
}

pub trait AgentHook: Send + Unpin {
    type Producer: Iterator<Item = rRequest> + Send;
    fn on_start_hook(&mut self) -> NextRequest;
    fn on_step_hook(&mut self, response: Response) -> NextRequest;
    fn on_close_hook(&mut self);
    fn build_producer() -> Self::Producer;
}
