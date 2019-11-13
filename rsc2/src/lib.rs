pub mod agent;

mod commands;
pub use crate::commands::*;

mod coordinator;
pub use coordinator::Coordinator;

pub mod builder {
    use super::{agent, Commands};
    use rsc2_pb::{prelude::*, sc2_api};

    pub struct RawGame {
        message: Vec<Commands>,
    }
    impl RawGame {
        pub fn new(
            _agent: Box<dyn agent::Agent>,
            create: sc2_api::RequestCreateGame,
            join: sc2_api::RequestJoinGame,
        ) -> Self {
            let mut message = Vec::with_capacity(5);
            message.push(Commands::Launched {});
            message.push(Commands::CreateGame { request: create });
            message.push(Commands::JoinGame { request: join });
            message.push(Commands::LeaveGame {});
            message.push(Commands::QuitGame {});
            Self { message }
        }
    }
    impl IntoIterator for RawGame {
        type Item = Commands;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.message.into_iter()
        }
    }

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
