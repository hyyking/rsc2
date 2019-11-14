pub mod agent;

mod commands;
pub use crate::commands::*;

mod coordinator;
pub use coordinator::Coordinator;

pub use rsc2_pb::prelude as pb_prelude;
pub use rsc2_pb::sc2_api;

pub mod builder {
    use super::{agent, Commands};
    use rsc2_pb::sc2_api;

    pub struct RawRequestGame<T>
    where
        T: agent::Agent,
    {
        message: Vec<Commands<T>>,
    }
    impl<T> RawRequestGame<T>
    where
        T: agent::Agent,
    {
        pub fn new(
            agent: T,
            create: sc2_api::RequestCreateGame,
            join: sc2_api::RequestJoinGame,
        ) -> Self {
            let mut message = Vec::with_capacity(5);
            message.push(Commands::Launched {});
            message.push(Commands::CreateGame { request: create });
            message.push(Commands::JoinGame {
                request: join,
                agent,
            });
            message.push(Commands::LeaveGame {});
            message.push(Commands::QuitGame {});
            Self { message }
        }
    }
    impl<T> IntoIterator for RawRequestGame<T>
    where
        T: agent::Agent,
    {
        type Item = Commands<T>;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.message.into_iter()
        }
    }
}
