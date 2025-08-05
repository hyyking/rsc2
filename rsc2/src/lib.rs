#[macro_use]
extern crate log;

use std::io;
use std::net::ToSocketAddrs;

use rsc2_pb::codec::S2Codec;
pub use rsc2_pb::protocol;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio_util::codec::FramedParts;
use websocket_lite::ClientBuilder;

pub mod definitions;
mod ingame;
pub mod prelude;
pub mod state_machine;

use crate::definitions::ToMapRef;
pub use crate::state_machine::Core;
pub use crate::state_machine::InGame;
pub use ingame::InGameListener;

pub type Connection = Framed<TcpStream, S2Codec>;

pub async fn connect_s2api(addr: impl std::net::ToSocketAddrs) -> io::Result<Connection> {
    let addr = format!(
        "ws://{}/sc2api",
        addr.to_socket_addrs()
            .ok()
            .as_mut()
            .and_then(Iterator::next)
            .unwrap()
    );
    let client = ClientBuilder::new(&addr).map(ClientBuilder::async_connect_insecure);
    match client {
        Ok(client) => match client.await {
            Ok(framed) => {
                let FramedParts { io, codec, .. } = framed.into_parts();
                Ok(Framed::from_parts(FramedParts::new::<
                    rsc2_pb::protocol::Request,
                >(
                    io, S2Codec::from(codec)
                )))
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::ConnectionRefused, e)),
        },
        Err(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid address",
        )),
    }
}

/// Create a game with the given players and map.
/// This function will connect to the SC2 API and create a game with the specified players and map.
/// It returns a tuple containing the game state and the connection to the SC2 API.
///
/// # Arguments
/// * `core` - A mutable reference to the core state machine.
/// * `addr` - The address of the SC2 API server.
/// * `players` - An iterator over the players to be added to the game.
/// * `map` - A reference to the map to be used for the game.
/// * `realtime` - A boolean indicating whether the game should be played in realtime or not.
///
pub async fn create_game<'core, P: Into<protocol::PlayerSetup>>(
    core: &'core mut Core,
    addr: impl ToSocketAddrs,
    players: impl IntoIterator<Item = P>,
    map: impl ToMapRef,
    realtime: bool,
) -> io::Result<(InGame<'core>, Connection)> {
    let players: Vec<protocol::PlayerSetup> = players.into_iter().map(Into::into).collect();

    let participant_race = players
        .iter()
        .find(|p| (p.r#type == Some(protocol::PlayerType::Participant as i32)))
        .map(|p| p.race)
        .flatten();

    // game assumed to be running
    let state = core
        .launched()
        .ok_or(io::Error::from(io::ErrorKind::Interrupted))?;

    let mut connection = connect_s2api(addr).await?;

    // create game request
    let mut create_game = protocol::RequestCreateGame::default();
    create_game.player_setup = players;

    create_game.map = Some(map.to_map());
    create_game.realtime = Some(realtime);

    let state = state
        .create_game(&mut connection, create_game)
        .await?
        .ok_or(io::Error::from(io::ErrorKind::Interrupted))?;

    let mut join_game = protocol::RequestJoinGame::default();
    join_game.participation =
        participant_race.map(protocol::request_join_game::Participation::Race);
    join_game.options = Some(protocol::InterfaceOptions {
        raw: Some(true),
        ..Default::default()
    });
    let state = state.join_game(&mut connection, join_game).await?.unwrap();

    Ok((state, connection))
}
