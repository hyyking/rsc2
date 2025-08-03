use std::ffi::OsStr;

use rsc2_pb::protocol;
pub use rsc2_pb::protocol::Difficulty;
pub use rsc2_pb::protocol::Race;

pub use crate::Connection;
pub use crate::connect_s2api;
pub use crate::state_machine::Core;
pub use crate::state_machine::InGame;

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

fn from_path(path: &std::path::Path) -> protocol::request_create_game::Map {
    if path.exists() && path.is_file() {
        protocol::request_create_game::Map::LocalMap(protocol::LocalMap {
            map_path: Some(path.to_string_lossy().into()),
            map_data: None,
        })
    } else {
        protocol::request_create_game::Map::BattlenetMapName(path.to_string_lossy().into())
    }
}

pub trait ToMapRef {
    fn to_map(self) -> protocol::request_create_game::Map;
}

impl ToMapRef for protocol::request_create_game::Map {
    fn to_map(self) -> protocol::request_create_game::Map {
        self
    }
}

impl<'a> ToMapRef for &'a str {
    fn to_map(self) -> protocol::request_create_game::Map {
        from_path(std::path::Path::new(self))
    }
}

impl ToMapRef for String {
    fn to_map(self) -> protocol::request_create_game::Map {
        self.as_str().to_map()
    }
}

impl ToMapRef for &OsStr {
    fn to_map(self) -> protocol::request_create_game::Map {
        from_path(std::path::Path::new(self))
    }
}

impl ToMapRef for &std::path::Path {
    fn to_map(self) -> protocol::request_create_game::Map {
        from_path(self)
    }
}
