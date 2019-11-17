use crate::agent::{AgentHook, NextRequest};
use rsc2_pb::sc2_api::{RequestGameInfo, Response};
use std::marker::Unpin;

pub trait RawAgent: Send + Unpin {
    fn on_response(&mut self, response: &Response) -> NextRequest;
}

pub struct NewRawAgent<A: RawAgent>(pub A);
impl<A> AgentHook for NewRawAgent<A>
where
    A: RawAgent,
{
    fn on_start_hook(&mut self) -> NextRequest {
        log::trace!("On start hook");
        NextRequest::Agent(RequestGameInfo::default().into())
    }
    fn on_step_hook(&mut self, response: &Response) -> NextRequest {
        log::trace!("On step hook");
        self.0.on_response(response)
    }
    fn on_close_hook(&mut self) {
        log::trace!("On close hook");
    }
}
