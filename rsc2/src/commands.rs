#[derive(Clone, Copy)]
pub enum Commands {
    /// Game has already been launched
    Launched {},
    /// Create a new game
    CreateGame {},
    /// Join an existing game
    JoinGame {},
    /// Start a replay
    StartReplay {},
    /// Restart a game
    RestartGame {},
    /// Leave the game but keep the instance running
    LeaveGame {},
}

impl From<&Commands> for Commands {
    fn from(other: &Self) -> Self {
        other.clone()
    }
}
