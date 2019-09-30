use rsc2_pb::sc2_api;

// use tokio::prelude::Future;
// pub type RequestFuture = Box<dyn Future<Item = Request, Error = ()>>;

pub trait Configurable {
    fn bot_config(&self) -> BotConfig;
}

pub trait Bot: Configurable {
    fn on_start(&mut self, _info: sc2_api::ResponseGameInfo) -> Option<sc2_api::Request> {
        None
    }
    fn on_step(&mut self, info: sc2_api::Observation, tick: u32) -> Option<sc2_api::Request>;
    fn on_end(&mut self) -> Option<sc2_api::Request> {
        None
    }
}

pub struct BotConfig {
    pub race: sc2_api::Race,
}
