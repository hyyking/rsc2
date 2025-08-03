use std::io;
use std::net::ToSocketAddrs;

use rsc2_pb::protocol;
pub use rsc2_pb::protocol::Difficulty;
pub use rsc2_pb::protocol::Race;

use crate::Connection;
use crate::connect;
use crate::state_machine::Core;
use crate::state_machine::InGame;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Player<T>
where
    T: Into<String>,
{
    name: T,
    race: Race,
    kind: protocol::PlayerType,
    difficulty: Option<Difficulty>,
}

impl<T> Player<T>
where
    T: Into<String>,
{
    pub fn participant(name: T, race: Race) -> Self {
        Self {
            name,
            race,
            kind: protocol::PlayerType::Participant,
            difficulty: None,
        }
    }
    pub fn bot(name: T, race: Race, difficulty: Difficulty) -> Self {
        Self {
            name,
            race,
            kind: protocol::PlayerType::Computer,
            difficulty: Some(difficulty),
        }
    }
}

impl<T> Into<protocol::PlayerSetup> for Player<T>
where
    T: Into<String>,
{
    fn into(self) -> protocol::PlayerSetup {
        let mut s = protocol::PlayerSetup {
            player_name: Some(self.name.into()),
            ..Default::default()
        };
        s.set_type(self.kind);
        s.set_race(self.race);
        if let Some(difficulty) = self.difficulty {
            s.set_difficulty(difficulty);
        }
        s
    }
}

fn local_map(path: impl Into<String>) -> protocol::request_create_game::Map {
    protocol::request_create_game::Map::LocalMap(protocol::LocalMap {
        map_path: Some(path.into()),
        map_data: None,
    })
}

pub async fn create_game<'core, P: Into<protocol::PlayerSetup>>(
    core: &'core mut Core,
    addr: impl ToSocketAddrs,
    players: impl IntoIterator<Item = P>,
    map: impl Into<String>,
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

    let mut connection = connect(addr).await?;

    // create game request
    let mut create_game = protocol::RequestCreateGame::default();
    create_game.player_setup = players;
    create_game.map = Some(local_map(map));
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
