use crate::states::{Ended, IsProtocolState, ProtocolStateMachine};
use rsc2_pb::{prelude::*, sc2_api};

#[derive(Debug, Default)]
pub struct InGame; // InGame info

impl IsProtocolState for InGame {
    fn gamestate_request(&self, game_loop: u32) -> EncodeResult {
        sc2_api::Request::with_id(sc2_api::RequestObservation::nofog(game_loop), game_loop).into()
    }
}

/// InGame will either end the current game or go to the next step (different modes: Step/RealTime)
// Transitions:
//      InGame -> Ended
impl From<ProtocolStateMachine<InGame>> for ProtocolStateMachine<Ended> {
    fn from(prev: ProtocolStateMachine<InGame>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: Ended {},
        }
    }
}
