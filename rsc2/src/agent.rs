use rsc2_pb::sc2_api;

pub struct AgentConfig {
    pub race: sc2_api::Race,
}

pub trait Agent {
    fn config(&self) -> AgentConfig;
    fn on_step(&mut self, info: sc2_api::Observation, tick: u32) -> Option<sc2_api::Request>;

    fn on_start(&mut self, _info: sc2_api::ResponseGameInfo) -> Option<sc2_api::Request> {
        None
    }
    fn on_end(&mut self) -> Option<sc2_api::Request> {
        None
    }
}
