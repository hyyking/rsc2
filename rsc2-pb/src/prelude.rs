use crate::sc2_api;

pub use crate::default::*;
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

impl sc2_api::RequestJoinGame {
    pub fn with_race(race: sc2_api::Race) -> Self {
        use sc2_api::request_join_game::Participation;
        Self {
            participation: Some(Participation::Race(race as i32)),
            ..Self::default_config()
        }
    }
}

impl sc2_api::RequestObservation {
    pub fn nofog(game_loop: u32) -> Self {
        Self {
            disable_fog: Some(false),
            game_loop: Some(game_loop),
        }
    }
}

impl sc2_api::Request {
    pub fn with_id<M>(req: M, id: u32) -> Self
    where
        M: Into<sc2_api::request::Request>,
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

impl sc2_api::InterfaceOptions {
    pub fn raw_mode() -> Self {
        Self {
            raw: Some(true),
            score: Some(true),
            feature_layer: None,
            render: None,

            /// By default cloaked units are completely hidden. This shows some details.
            show_cloaked: Some(false),

            /// By default burrowed units are completely hidden. This shows some details for those that produce a shadow.
            show_burrowed_shadows: Some(false),

            /// Return placeholder units (buildings to be constructed), both for raw and feature layers.
            show_placeholders: Some(true),

            /// By default raw actions select, act and revert the selection. This is useful
            /// if you're playing simultaneously with the agent so it doesn't steal your
            /// selection. This inflates APM (due to deselect) and makes the actions hard
            /// to follow in a replay. Setting this to true will cause raw actions to do
            /// select, act, but not revert the selection.
            raw_affects_selection: Some(true),

            /// Changes the coordinates in raw.proto to be relative to the playable area.
            /// The map_size and playable_area will be the diagonal of the real playable area.
            raw_crop_to_playable_area: Some(false),
        }
    }
    /* fn feature_layer() -> Self {
        Self {
            raw: Some(true),
            score: Some(true),
            feature_layer: Some(sc2_api::SpatialCameraSetup {
                resolution: Some(sc2_api::Size2Di {
                    x: Some(0),
                    y: Some(0),
                }),
                minimap_resolution: Some(sc2_api::Size2Di {
                    x: Some(0),
                    y: Some(0),
                }),
                width: Some(100_f32),
                crop_to_playable_area: Some(true),
                allow_cheating_layers: Some(true),
            }),
            render: Some(sc2_api::SpatialCameraSetup {
                resolution: Some(sc2_api::Size2Di {
                    x: Some(0),
                    y: Some(0),
                }),
                minimap_resolution: Some(sc2_api::Size2Di {
                    x: Some(0),
                    y: Some(0),
                }),
                width: Some(100_f32),
                crop_to_playable_area: Some(true),
                allow_cheating_layers: Some(true),
            }),
            show_cloaked: Some(false),
            show_burrowed_shadows: Some(false),
            show_placeholders: Some(false),
            raw_affects_selection: Some(true),
            raw_crop_to_playable_area: Some(true),
        }
    } */
}

impl sc2_api::RequestStartReplay {
    pub fn from_file<T: Into<String>>(file: T) -> Self {
        Self {
            map_data: None,
            observed_player_id: None,
            options: None,
            disable_fog: None,
            realtime: None,
            record_replay: None,
            replay: Some(sc2_api::request_start_replay::Replay::ReplayPath(
                file.into(),
            )),
        }
    }
}
