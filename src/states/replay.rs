use crate::states::{IsProtocolState, ProtocolStateMachine, Ended};

pub struct InReplay; // InReplay info
impl IsProtocolState for InReplay {}

/// InReplay will either end or go to the next step (different modes: Step/RealTime)
// Transitions
//      InReplay -> Ended
impl From<ProtocolStateMachine<InReplay>> for ProtocolStateMachine<Ended> {
    fn from(prev: ProtocolStateMachine<InReplay>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: Ended {},
        }
    }
}
