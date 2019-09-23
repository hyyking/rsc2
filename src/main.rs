#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate tokio;

extern crate websocket;
extern crate bytes;

extern crate prost;
#[macro_use]
extern crate prost_derive;

mod states;
mod sc2_api;

use states::{ProtocolArg, ProtocolState};

use websocket::{client::builder::ParseError, ClientBuilder};

fn main() -> Result<(), ParseError> {
    pretty_env_logger::init_timed();

    debug!("Establishing Connection");
    let established = ClientBuilder::new("ws://127.0.0.1:5000/sc2api")
        .unwrap()
        .async_connect_insecure();
//       ;

    debug!("Connection Established to ws://127.0.0.1:5000/sc2api");
    let engine: ProtocolState = established.into();
    let next_state = engine.run(ProtocolArg::CreateGame);
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
