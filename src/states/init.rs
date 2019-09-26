use crate::states::{InGame, IsProtocolState, ProtocolStateMachine};

#[derive(Debug)]
pub struct InitGame; // InitGame info

impl IsProtocolState for InitGame {
    fn join_game_request(&self) {}
}

/// InitGame will transition in InGame
// Transitions:
//      InitGame -> InGame
impl From<ProtocolStateMachine<InitGame>> for ProtocolStateMachine<InGame> {
    fn from(prev: ProtocolStateMachine<InitGame>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InGame {},
        }
    }
}
