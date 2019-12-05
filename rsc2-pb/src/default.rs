use crate::api;

/// Default config for fast setup
pub trait DefaultConfig {
    /// Build a struct using it's default config
    fn default_config() -> Self;
}

impl DefaultConfig for Vec<api::PlayerSetup> {
    fn default_config() -> Self {
        // Player vs Terran Easy Ai
        vec![
            api::PlayerSetup::player(),
            api::PlayerSetup::custom_bot(
                api::Race::Terran,
                api::Difficulty::Easy,
                api::AiBuild::RandomBuild,
            ),
        ]
    }
}

impl DefaultConfig for api::request_create_game::Map {
    fn default_config() -> Self {
        api::request_create_game::Map::LocalMap(api::LocalMap {
            map_path: Some("Ladder2017Season1/AbyssalReefLE.SC2Map".into()),
            map_data: None,
        })
    }
}

impl DefaultConfig for api::RequestCreateGame {
    fn default_config() -> Self {
        Self {
            player_setup: Vec::default_config(),
            disable_fog: Some(false),
            random_seed: None,
            realtime: Some(true),
            map: Some(api::request_create_game::Map::default_config()),
        }
    }
}

impl DefaultConfig for api::InterfaceOptions {
    fn default_config() -> Self {
        Self::raw_mode()
    }
}

impl DefaultConfig for api::RequestJoinGame {
    fn default_config() -> Self {
        Self {
            participation: Some(api::request_join_game::Participation::Race(
                api::Race::Terran as i32,
            )),
            options: Some(api::InterfaceOptions::default_config()),
            server_ports: None,
            client_ports: vec![], /* vec![api::PortSet {game_port: Some(5000), base_port: Some(5000),}], */
            shared_port: None,
            player_name: None,
            host_ip: None,
        }
    }
}
