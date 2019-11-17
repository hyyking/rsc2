use std::marker::Unpin;

use crate::agent::{AgentHook, NextRequest};

use rsc2_pb::sc2_api::{
    response::Response as rResponse, RequestAction, Response, ResponseGameInfo, ResponseObservation,
};

pub trait Agent: Send + Unpin {
    fn on_start(&mut self, _info: &ResponseGameInfo) -> Option<RequestAction> {
        None
    }

    fn on_step(&mut self, info: &ResponseObservation) -> Option<RequestAction>;

    fn on_end(&mut self) {}
}

pub struct NewAgent<A: Agent>(pub A);
impl<A> AgentHook for NewAgent<A>
where
    A: Agent,
{
    fn on_start_hook(&mut self) -> NextRequest {
        log::trace!("On start hook");
        // NextRequest::Agent(RequestGameInfo::default().into())
        NextRequest::Observation
    }
    fn on_step_hook(&mut self, response: &Response) -> NextRequest {
        log::trace!("On step hook");
        let Response { response, .. } = response;
        match response.as_ref() {
            Some(rResponse::Observation(robs)) => self.0.on_step(robs).map(|r| r.into()).into(),
            Some(rResponse::GameInfo(robs)) => self.0.on_start(robs).map(|r| r.into()).into(),
            _ => NextRequest::Observation,
        }
    }
    fn on_close_hook(&mut self) {
        log::trace!("On close hook");
        self.0.on_end()
    }
}
