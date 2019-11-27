use crate::hook;
use rsc2_pb::api as pb;
use std::net::SocketAddrV4;

#[derive(Clone)]
pub enum Commands<A>
where
    A: hook::AgentHook,
{
    /// Game has already been launched
    Launched { socket: SocketAddrV4 },
    /// Create a new game
    CreateGame { request: pb::RequestCreateGame },
    /// Join an existing game
    JoinGame {
        agent: A,
        request: pb::RequestJoinGame,
    },
    /// Start a replay
    StartReplay {
        agent: A,
        request: pb::RequestStartReplay,
    },
    /// Restart a game
    RestartGame,
    /// Leave the game but keep the instance running
    LeaveGame,
    /// Quit the running instance of the game
    QuitGame,
}

impl<A> From<&Commands<A>> for Commands<A>
where
    A: Clone + hook::AgentHook,
{
    fn from(other: &Self) -> Self {
        other.clone()
    }
}
