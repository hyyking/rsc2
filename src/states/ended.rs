use crate::states::{IsProtocolState, ProtocolStateMachine, Launched, InGame};

pub struct Ended; // Ended info

impl IsProtocolState for Ended {}

impl Ended {
    fn restart_game(&self) {} 
    fn close_game(&self) {} 
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
