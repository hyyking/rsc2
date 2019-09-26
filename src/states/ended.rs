use crate::states::{InGame, IsProtocolState, Launched, ProtocolStateMachine};

#[derive(Debug)]
pub struct Ended; // Ended info

impl IsProtocolState for Ended {
    fn restart_game_request(&self) {}
    fn close_game_request(&self) {}
}

/// Once a game has ended we can either replay it, launch another one or end
// Transitions:
//      Ended -> Launched
//      Ended -> InGame
impl From<ProtocolStateMachine<Ended>> for ProtocolStateMachine<Launched> {
    fn from(prev: ProtocolStateMachine<Ended>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: Launched {},
        }
    }
}

impl From<ProtocolStateMachine<Ended>> for ProtocolStateMachine<InGame> {
    fn from(prev: ProtocolStateMachine<Ended>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InGame {},
        }
    }
}
