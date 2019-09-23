use crate::states::{IsProtocolState, ProtocolStateMachine, SharedState, InGame};

pub struct InitGame; // InitGame info

impl IsProtocolState for InitGame {
    fn join_game(&mut self, shared: &mut SharedState) {}
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
