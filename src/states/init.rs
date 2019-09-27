use crate::proto::{prelude::*, sc2_api};
use crate::states::{InGame, IsProtocolState, ProtocolStateMachine};

#[derive(Debug)]
pub struct InitGame; // InitGame info

impl IsProtocolState for InitGame {
    fn join_game_request(&self) -> EncodeResult {
        sc2_api::Request::with_id(sc2_api::RequestJoinGame::default_config(), 1).into()
    }
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
