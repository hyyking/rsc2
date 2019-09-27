use crate::proto::sc2_api;

pub use crate::proto::default::*;
pub use crate::proto::result::*;
pub use crate::proto::wrap::*;

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

#[allow(dead_code)]
impl sc2_api::PlayerSetup {
    // OBSERVER ---------------

    /// Add a default observer
    pub fn observer() -> Self {
        Self {
            r#type: Some(sc2_api::PlayerType::Observer as i32), // Observer
            ..Default::default()
        }
    }

    // HUMAN/SCRIPTED ---------------

    /// Add a default player
    pub fn player() -> Self {
        Self {
            r#type: Some(sc2_api::PlayerType::Participant as i32), // Player
            ..Default::default()                                   // The rest to None
        }
    }
    /// Add a default player with race
    pub fn player_with_race(race: sc2_api::Race) -> Self {
        Self {
            race: Some(race as i32),
            ..Self::player() // The rest to player()
        }
    }

    // BOT ---------------

    /// Add a custom bot to the Vec<PlayerSetup>
    pub fn custom_bot(r: sc2_api::Race, d: sc2_api::Difficulty, b: sc2_api::AiBuild) -> Self {
        Self {
            r#type: Some(sc2_api::PlayerType::Computer as i32), // Bot
            race: Some(r as i32),
            difficulty: Some(d as i32),              //Easy
            player_name: Some("SpectreVert".into()), // NOTE: find a better name
            ai_build: Some(b as i32),                // RandomBuild?
        }
    }

    /// Add a randomized bot to the Vec<PlayerSetup>
    pub fn random_bot() -> Self {
        Self::custom_bot(
            sc2_api::Race::Random,
            sc2_api::Difficulty::Easy,
            sc2_api::AiBuild::RandomBuild,
        )
    }
}
