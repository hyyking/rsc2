use crate::states::{IsProtocolState, ProtocolStateMachine, InGame};

pub struct InitGame; // InitGame info

impl IsProtocolState for InitGame {}

impl InitGame {
    fn join_game(&self) {}
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
