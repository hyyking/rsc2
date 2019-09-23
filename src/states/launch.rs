use crate::states::{IsProtocolState, ProtocolStateMachine, InitGame, InGame, InReplay};

pub struct Launched; // Launched info

impl IsProtocolState for Launched {}

impl Launched {
    fn create_game(&self) {}
    fn join_game(&self) {}
    fn start_replay(&self) {}
}

impl Default for Launched {
    fn default() -> Self {
        return Launched {};
    }
}

/// Launched State launches a SC2 game instance and can transition in either a GameCreation state
/// or a Playing/Spectating state
// Transitions:
//      Launched -> InitGame
//      Launched -> InGame
//      Launched -> InReplay

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InitGame> {
    fn from(prev: ProtocolStateMachine<Launched>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InitGame {},
        }
    }
}

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InGame> {
    fn from(prev: ProtocolStateMachine<Launched>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InGame {},
        }
    }
}

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InReplay> {
    fn from(prev: ProtocolStateMachine<Launched>) -> Self {
        ProtocolStateMachine {
            // Shared Values
            shared: prev.shared,
            inner: InReplay {},
        }
    }
}
