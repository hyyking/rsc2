extern crate log;
extern crate pretty_env_logger;

extern crate rsc2;

use log::debug;

use rsc2::agent::{Agent, AgentConfig};
use rsc2::engine::{ProtocolArg, ProtocolState};
use rsc2::sc2_api;
use rsc2::websocket::{client::builder::ParseError, ClientBuilder};

struct MyBot;
impl Agent for MyBot {
    fn on_step(&mut self, _info: sc2_api::Observation, tick: u32) -> Option<sc2_api::Request> {
        println!("{:?}", tick);
        Some(sc2_api::Request::with_id(
            sc2_api::RequestQuit::default(),
            42,
        ))
    }
    fn config(&self) -> AgentConfig {
        AgentConfig {
            race: sc2_api::Race::Zerg,
        }
    }
}

#[allow(unused_variables)]
fn main() -> Result<(), ParseError> {
    pretty_env_logger::init_timed();

    let bot = Box::new(MyBot {});

    debug!("Establishing Connection");
    let connection = ClientBuilder::new("ws://127.0.0.1:5000/sc2api")?.async_connect_insecure();
    let base: ProtocolState = connection.into();
    debug!("Connection Established to ws://127.0.0.1:5000/sc2api");
    let create_game = base.run(ProtocolArg::CreateGame);
    let join_game = create_game.run(ProtocolArg::JoinGame(bot));
    let play_game = join_game.run(ProtocolArg::PlayGame);

    Ok(())
}
