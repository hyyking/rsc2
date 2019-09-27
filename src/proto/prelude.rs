use crate::proto::sc2_api;

pub use crate::proto::default::*;
pub use crate::proto::result::*;
pub use prost::Message;

/* pub enum Request {
    CreateGame(RequestCreateGame),
    JoinGame(RequestJoinGame),
    RestartGame(RequestRestartGame),
    StartReplay(RequestStartReplay),
    LeaveGame(RequestLeaveGame),
    QuickSave(RequestQuickSave),
    QuickLoad(RequestQuickLoad),
    Quit(RequestQuit),
    GameInfo(RequestGameInfo),
    Observation(RequestObservation),
    Action(RequestAction),
    ObsAction(RequestObserverAction),
    Step(RequestStep),
    Data(RequestData),
    Query(RequestQuery),
    SaveReplay(RequestSaveReplay),
    MapCommand(RequestMapCommand),
    ReplayInfo(RequestReplayInfo),
    AvailableMaps(RequestAvailableMaps),
    SaveMap(RequestSaveMap),
    Ping(RequestPing),
    Debug(RequestDebug),
} */

impl sc2_api::Request {
    pub fn with_id<M>(req: M, id: u32) -> Self
    where
        M: Message + Into<sc2_api::request::Request>,
    {
        Self {
            id: Some(id),
            request: Some(req.into()),
        }
    }
}

impl sc2_api::PlayerSetup {
    pub fn player() -> Self {
        Self {
            r#type: Some(sc2_api::PlayerType::Participant as i32), // Player
            ..Default::default()                                   // The rest to None
        }
    }

    #[allow(dead_code)]
    pub fn bot() -> Self {
        Self {
            r#type: Some(sc2_api::PlayerType::Participant as i32), // Player
            race: Some(sc2_api::Race::Random as i32),
            difficulty: Some(sc2_api::Difficulty::Easy as i32),
            player_name: Some("SpectreVert".into()),
            ai_build: Some(sc2_api::AiBuild::RandomBuild as i32),
        }
    }

    #[allow(dead_code)]
    pub fn player_with_race(race: sc2_api::Race) -> Self {
        Self {
            r#type: Some(sc2_api::PlayerType::Participant as i32), // Player
            race: Some(race as i32),
            ..Default::default() // The rest to None
        }
    }

    pub fn default_bot_with_race(race: sc2_api::Race) -> Self {
        Self {
            r#type: Some(sc2_api::PlayerType::Computer as i32), // Player
            race: Some(race as i32),
            difficulty: Some(sc2_api::Difficulty::Easy as i32),
            player_name: Some("SpectreVert".into()),
            ai_build: Some(sc2_api::AiBuild::RandomBuild as i32),
        }
    }
}

impl Into<sc2_api::request::Request> for sc2_api::RequestPing {
    fn into(self) -> sc2_api::request::Request {
        sc2_api::request::Request::Ping(self)
    }
}

impl Into<sc2_api::request::Request> for sc2_api::RequestCreateGame {
    fn into(self) -> sc2_api::request::Request {
        sc2_api::request::Request::CreateGame(self)
    }
}

impl Into<sc2_api::request_create_game::Map> for sc2_api::LocalMap {
    fn into(self) -> sc2_api::request_create_game::Map {
        sc2_api::request_create_game::Map::LocalMap(self)
    }
}
