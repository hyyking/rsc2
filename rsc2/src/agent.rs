use rsc2_pb::sc2_api;

pub trait Agent: Send + Sync {
    fn on_step(&mut self, info: &sc2_api::Observation) -> Option<sc2_api::RequestAction>;

    fn on_start(&mut self, _info: sc2_api::ResponseGameInfo) -> Option<sc2_api::RequestAction> {
        None
    }
    fn on_end(&mut self) -> Option<sc2_api::Request> {
        None
    }
}
