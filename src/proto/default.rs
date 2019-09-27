use crate::proto::sc2_api;

pub trait DefaultConfig {
    fn default_config() -> Self;
}

impl DefaultConfig for Vec<sc2_api::PlayerSetup> {
    fn default_config() -> Self {
        /// Terran vs Terran Ai
        vec![
            sc2_api::PlayerSetup::player(),
            sc2_api::PlayerSetup::custom_bot(
                sc2_api::Race::Terran,
                sc2_api::Difficulty::Easy,
                sc2_api::AiBuild::RandomBuild,
            ),
        ]
    }
}

impl DefaultConfig for sc2_api::request_create_game::Map {
    fn default_config() -> Self {
        sc2_api::LocalMap {
            map_path: Some("Ladder2017Season1/AbyssalReefLE.SC2Map".into()),
            map_data: None,
        }
        .into()
    }
}

impl DefaultConfig for sc2_api::RequestCreateGame {
    fn default_config() -> Self {
        Self {
            /// Player vs Bot
            player_setup: Vec::default_config(),
            /// Fog is Enabled by Default
            disable_fog: Some(false),
            /// Seed is random see https://github.com/Blizzard/s2client-proto/blob/master/docs/protocol.md#randomness
            random_seed: None,
            /// Realtime mode by default
            realtime: Some(true),
            /// $SC2_PATH/StarCraftII/Maps/Ladder2017Season1/AbyssalReefLE.SC2Map
            map: Some(sc2_api::request_create_game::Map::default_config()),
        }
    }
}
