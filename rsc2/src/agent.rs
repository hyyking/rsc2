use rsc2_pb::sc2_api::{request::Request as rRequest, RequestAction, RequestGameInfo};
use rsc2_pb::sc2_api::{
    response::Response as rResponse, Response, ResponseGameInfo, ResponseObservation,
};
use std::marker::Unpin;

pub trait AgentHook: Send + Sync + Unpin {
    fn on_start_hook(&mut self) -> Option<rRequest>;
    fn on_step_hook(&mut self, response: &Response) -> Option<rRequest>;
    fn on_close_hook(&mut self);
}

pub trait Agent: Send + Sync + Unpin {
    fn on_step(&mut self, info: &ResponseObservation) -> Option<RequestAction>;

    fn on_start(&mut self, _info: &ResponseGameInfo) -> Option<RequestAction> {
        None
    }
    fn on_end(&mut self) {}
}
pub struct NewAgent<A: Agent>(pub A);
impl<A> AgentHook for NewAgent<A>
where
    A: Agent,
{
    fn on_start_hook(&mut self) -> Option<rRequest> {
        Some(RequestGameInfo {}.into())
    }
    fn on_step_hook(&mut self, response: &Response) -> Option<rRequest> {
        let Response { response, .. } = response;
        match response.as_ref()? {
            rResponse::Observation(robs) => self.0.on_step(robs).map(|r| r.into()),
            rResponse::GameInfo(robs) => self.0.on_start(robs).map(|r| r.into()),
            _ => None,
        }
    }
    fn on_close_hook(&mut self) {
        self.0.on_end()
    }
}

pub trait RawAgent: Send + Sync + Unpin {
    fn on_response(&mut self, response: &Response) -> Option<rRequest>;
}

pub struct NewRawAgent<A: RawAgent>(pub A);
impl<A> AgentHook for NewRawAgent<A>
where
    A: RawAgent,
{
    fn on_start_hook(&mut self) -> Option<rRequest> {
        Some(RequestGameInfo {}.into())
    }
    fn on_step_hook(&mut self, response: &Response) -> Option<rRequest> {
        self.0.on_response(response)
    }
    fn on_close_hook(&mut self) {}
}
