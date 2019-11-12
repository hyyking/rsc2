use rsc2_pb::sc2_api;

#[derive(Clone)]
pub enum Commands {
    /// Game has already been launched
    Launched {},
    /// Create a new game
    CreateGame {
        request: sc2_api::RequestCreateGame,
    },
    /// Join an existing game
    JoinGame {
        request: sc2_api::RequestJoinGame,
    },
    /// Start a replay
    StartReplay {
        request: sc2_api::RequestStartReplay,
    },
    /// Restart a game
    RestartGame {},
    /// Leave the game but keep the instance running
    LeaveGame {},
    QuitGame {},
}

impl From<&Commands> for Commands {
    fn from(other: &Self) -> Self {
        other.clone()
    }
}
