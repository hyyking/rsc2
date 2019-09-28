use crate::states::{InGame, InReplay, InitGame, IsProtocolState, ProtocolStateMachine};

use rsc2_pb::{prelude::*, sc2_api};

#[derive(Debug)]
pub struct Launched; // Launched info

impl IsProtocolState for Launched {
    fn create_game_request(&self) -> EncodeResult {
        sc2_api::Request::with_id(sc2_api::RequestCreateGame::default_config(), 0).into()
    }
    fn join_game_request(&self) -> EncodeResult {
        sc2_api::Request::with_id(sc2_api::RequestJoinGame::default_config(), 1).into()
    }
    fn start_replay_request(&self) {}
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
            shared: prev.shared,
            inner: InReplay {},
        }
    }
}
