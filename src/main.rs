extern crate log;
extern crate pretty_env_logger;

extern crate rsc2;

use log::debug;
use rsc2::states::{ProtocolArg, ProtocolState};
use rsc2::websocket::{client::builder::ParseError, ClientBuilder};

#[allow(unused_variables)]
fn main() -> Result<(), ParseError> {
    pretty_env_logger::init_timed();

    debug!("Establishing Connection");
    let connection = ClientBuilder::new("ws://127.0.0.1:5000/sc2api")?.async_connect_insecure();
    let base: ProtocolState = connection.into();

    debug!("Connection Established to ws://127.0.0.1:5000/sc2api");
    let create_game = base.run(ProtocolArg::CreateGame);
    let join_game = create_game.run(ProtocolArg::JoinGame);
    let play_game = join_game.run(ProtocolArg::PlayGame);

    Ok(())
}
