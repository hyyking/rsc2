extern crate log;
extern crate pretty_env_logger;

extern crate rsc2;

use log::debug;

use rsc2::bot::{Bot, BotConfig, Configurable};
use rsc2::engine::{ProtocolArg, ProtocolState};
use rsc2::sc2_api;
use rsc2::websocket::{client::builder::ParseError, ClientBuilder};

struct MyBot;

impl Bot for MyBot {
    fn on_step(&mut self, info: sc2_api::Observation, _tick: u32) -> Option<sc2_api::Request> {
        println!("{:?}", info);
        Some(sc2_api::Request::with_id(
            sc2_api::RequestQuit::default(),
            42,
        ))
    }
}

impl Configurable for MyBot {
    fn bot_config(&self) -> BotConfig {
        BotConfig {
            race: sc2_api::Race::Zerg,
        }
    }
}

#[allow(unused_variables)]
fn main() -> Result<(), ParseError> {
    pretty_env_logger::init_timed();

    debug!("Establishing Connection");
    let connection = ClientBuilder::new("ws://127.0.0.1:5000/sc2api")?.async_connect_insecure();
    let base: ProtocolState = connection.into();
    let bot = Box::new(MyBot {});
    debug!("Connection Established to ws://127.0.0.1:5000/sc2api");
    let create_game = base.run(ProtocolArg::CreateGame);
    let join_game = create_game.run(ProtocolArg::JoinGame(bot));
    let play_game = join_game.run(ProtocolArg::PlayGame);

    Ok(())
}
