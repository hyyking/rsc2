use crate::agent;
use rsc2_pb::sc2_api;

#[derive(Clone)]
pub enum Commands<T>
where
    T: agent::Agent,
{
    /// Game has already been launched
    Launched {},
    /// Create a new game
    CreateGame {
        request: sc2_api::RequestCreateGame,
    },
    /// Join an existing game
    JoinGame {
        agent: T,
        request: sc2_api::RequestJoinGame,
    },
    /// Start a replay
    StartReplay {
        agent: T,
        request: sc2_api::RequestStartReplay,
    },
    /// Restart a game
    RestartGame {},
    /// Leave the game but keep the instance running
    LeaveGame {},
    QuitGame {},
}

impl<T> From<&Commands<T>> for Commands<T>
where
    T: Clone + agent::Agent,
{
    fn from(other: &Self) -> Self {
        other.clone()
    }
}
