use crate::states::{IsProtocolState, ProtocolStateMachine, Ended};

pub struct InGame; // InGame info

impl IsProtocolState for InGame {}

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
