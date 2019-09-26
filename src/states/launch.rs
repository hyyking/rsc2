use crate::sc2_api;
use crate::states::{InGame, InReplay, InitGame, IsProtocolState, ProtocolStateMachine};

use prost::{EncodeError, Message};

#[derive(Debug)]
pub struct Launched; // Launched info

impl IsProtocolState for Launched {
    fn create_game_request(&self) -> Result<websocket::OwnedMessage, EncodeError> {
        let mut buff = vec![];
        sc2_api::Request {
            id: None,
            request: Some(sc2_api::request::Request::Ping(sc2_api::RequestPing {})),
        }
        .encode(&mut buff)?;
        Ok(websocket::OwnedMessage::Binary(buff))
    }
    fn join_game_request(&self) {}
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
            // Shared Values
            shared: prev.shared,
            inner: InReplay {},
        }
    }
}
