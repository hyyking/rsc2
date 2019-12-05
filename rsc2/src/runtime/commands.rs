use crate::hook;
use rsc2_pb::api as pb;
use std::net::SocketAddrV4;

/// commands than can be fed in the coordinator to interact with a running instance.
#[derive(Clone, Debug)]
pub enum Commands<A: hook::AgentHook> {
    /// Launch a game or set it as launched. This call will attempt a connection to the Starcraft II instance.
    Launched {
        /// Address on wich the Starcraft II instance is running.
        socket: SocketAddrV4,
    },

    /// Create a new game from a request.
    CreateGame {
        /// initial request
        request: pb::RequestCreateGame,
    },

    /// Join an existing game with an agent.
    JoinGame {
        /// agent that will be run.
        agent: A,
        /// request to join the game.
        request: pb::RequestJoinGame,
    },
    /// Start a replay
    StartReplay {
        /// agent that will be run.
        agent: A,
        /// request to start the replay.
        request: pb::RequestStartReplay,
    },

    /// Restart a game
    RestartGame,

    /// Leave the current game but keep the instance running
    LeaveGame,

    /// Quit the instance of the game
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
