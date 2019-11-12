pub mod agent;

mod commands;
pub use crate::commands::*;

mod coordinator;
pub use coordinator::Coordinator;

pub mod builder {
    use super::Commands;
    use rsc2_pb::{prelude::*, sc2_api};

    pub struct MockGame;
    impl MockGame {
        pub fn new() -> Self {
            Self
        }
    }
    impl IntoIterator for MockGame {
        type Item = Commands;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            let rcg = sc2_api::RequestCreateGame::default_config();
            let rjg = sc2_api::RequestJoinGame::default_config();
            vec![
                Commands::Launched {},
                Commands::CreateGame { request: rcg },
                Commands::JoinGame { request: rjg },
                Commands::LeaveGame {},
                Commands::QuitGame {},
            ]
            .into_iter()
        }
    }
}
