extern crate pretty_env_logger;

use crate::states::{ProtocolArg, ProtocolState};
use crate::websocket::{client::builder::ParseError, ClientBuilder};

#[allow(unused_variables)]
fn main() -> Result<(), ParseError> {
    pretty_env_logger::init_timed();

    debug!("Establishing Connection");
    let connection = ClientBuilder::new("ws://127.0.0.1:5000/sc2api")?.async_connect_insecure();
    let base: ProtocolState = connection.into();

    debug!("Connection Established to ws://127.0.0.1:5000/sc2api");
    let create_game = base.run(ProtocolArg::CreateGame);
    let join_game = create_game.run(ProtocolArg::JoinGame);

    Ok(())
}
