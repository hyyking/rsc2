pub mod agent;
pub mod producer;

mod commands;
pub use crate::commands::*;

mod coordinator;
pub use coordinator::Coordinator;

pub use rsc2_pb::prelude as pb_prelude;
pub use rsc2_pb::sc2_api;

pub mod builder {
    use super::{agent, producer, Commands};
    use rsc2_pb::sc2_api;

    pub struct RawRequestGame<A>
    where
        A: agent::AgentHook + 'static,
    {
        messages: Vec<Commands<A, producer::RawProducer>>,
    }
    impl<T> RawRequestGame<T>
    where
        T: agent::AgentHook,
    {
        pub fn new(
            agent: T,
            create: sc2_api::RequestCreateGame,
            join: sc2_api::RequestJoinGame,
        ) -> Self {
            let mut messages = Vec::with_capacity(5);
            messages.push(Commands::Launched {
                socket: "127.0.0.1:5000".parse().unwrap(),
            });
            messages.push(Commands::CreateGame { request: create });
            messages.push(Commands::JoinGame {
                request: join,
                producer: producer::RawProducer::new(),
                agent,
            });
            messages.push(Commands::LeaveGame {});
            messages.push(Commands::QuitGame {});
            Self { messages }
        }
    }
    impl<T> IntoIterator for RawRequestGame<T>
    where
        T: agent::AgentHook,
    {
        type Item = Commands<T, producer::RawProducer>;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.messages.into_iter()
        }
    }
}
