use crate::sc2_api;
use crate::states::{
    InGame, InReplay, InitGame, IsProtocolState, ProtocolStateMachine, SharedState,
};

use bytes::Buf;
use std::borrow::BorrowMut;
use std::io::Cursor;

use prost::*;
use tokio::prelude::*;

pub struct Launched; // Launched info

impl IsProtocolState for Launched {
    fn create_game(&mut self, shared: &mut SharedState) {
        let mut buff = vec![];
        sc2_api::Request {
            id: None,
            request: Some(sc2_api::request::Request::Ping(sc2_api::RequestPing {})),
        }
        .encode(&mut buff)
        .unwrap();
        shared.conn.send(websocket::OwnedMessage::Binary(buff)).wait();
    }
    fn join_game(&mut self, shared: &mut SharedState) {}
    fn start_replay(&mut self, shared: &mut SharedState) {}
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
