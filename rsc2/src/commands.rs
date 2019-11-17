use crate::agent;
use rsc2_pb::sc2_api;
use sc2_api::request::Request as rRequest;
use std::marker::Unpin;
use std::net::SocketAddrV4;

#[derive(Clone)]
pub enum Commands<A, P>
where
    A: agent::AgentHook,
    P: Iterator<Item = rRequest> + Unpin,
{
    /// Game has already been launched
    Launched {
        socket: SocketAddrV4,
    },
    /// Create a new game
    CreateGame {
        request: sc2_api::RequestCreateGame,
    },
    /// Join an existing game
    JoinGame {
        agent: A,
        producer: P,
        request: sc2_api::RequestJoinGame,
    },
    /// Start a replay
    StartReplay {
        agent: A,
        producer: P,
        request: sc2_api::RequestStartReplay,
    },
    /// Restart a game
    RestartGame {},
    /// Leave the game but keep the instance running
    LeaveGame {},
    QuitGame {},
}

impl<A, P> From<&Commands<A, P>> for Commands<A, P>
where
    A: Clone + agent::AgentHook,
    P: Clone + Iterator<Item = rRequest> + Unpin,
{
    fn from(other: &Self) -> Self {
        other.clone()
    }
}
