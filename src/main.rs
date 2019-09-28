#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate bytes;
extern crate prost;
extern crate tokio;
extern crate websocket;

extern crate rsc2_pb;

mod states;

use states::{ProtocolArg, ProtocolState};

use websocket::{client::builder::ParseError, ClientBuilder};

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

/*
mod sc2_api;

use prost::*;
use tokio::prelude::*;
use websocket::OwnedMessage;

    let echo_future = new_connection()?
        .and_then(|(s, _)| {
            let mut buff = vec![];
            sc2_api::Request {
                id: None,
                request: Some(sc2_api::request::Request::Ping(sc2_api::RequestPing {})),
            }
            .encode(&mut buff)
            .unwrap();

            s.send(OwnedMessage::Binary(buff).into())
        })
        .and_then(|s| s.into_future().map_err(|e| e.0))
        .map(|(m, _)| match m.unwrap() {
            OwnedMessage::Binary(buff) => {
                println!("{:?}", sc2_api::Response::decode(buff).unwrap())
            }
            x => println!("{:?}", x),
        });
    runtime.block_on(echo_future).unwrap();
*/
