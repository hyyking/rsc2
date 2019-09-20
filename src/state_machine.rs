pub trait ProtocolState {}

pub struct ProtocolStateMachine<S>
where
    S: ProtocolState,
{
    // Add shared values here
    state: S,
}

pub struct Launched; // Launched info
impl ProtocolState for Launched {}

pub struct InitGame; // InitGame info
impl ProtocolState for InitGame {}

pub struct InGame; // InGame info
impl ProtocolState for InGame {}

pub struct InReplay; // InReplay info
impl ProtocolState for InReplay {}

pub struct Ended; // Ended info
impl ProtocolState for Ended {}

impl Default for ProtocolStateMachine<Launched> {
    fn default() -> Self {
        ProtocolStateMachine {
            // Share Values
            state: Launched {},
        }
    }
}

/// Launched State launches a SC2 game instance and can transition in either a GameCreation state
/// or a Playing/Spectating state
// Transitions:
//      Launched -> InitGame
//      Launched -> InGame
//      Launched -> InReplay

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InitGame> {
    fn from(_: ProtocolStateMachine<Launched>) -> ProtocolStateMachine<InitGame> {
        ProtocolStateMachine {
            // Shared Values
            state: InitGame {},
        }
    }
}

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InGame> {
    fn from(_: ProtocolStateMachine<Launched>) -> ProtocolStateMachine<InGame> {
        ProtocolStateMachine {
            // Shared Values
            state: InGame {},
        }
    }
}

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InReplay> {
    fn from(_: ProtocolStateMachine<Launched>) -> ProtocolStateMachine<InReplay> {
        ProtocolStateMachine {
            // Shared Values
            state: InReplay {},
        }
    }
}

/// InitGame will transition in InGame
// Transitions:
//      InitGame -> InGame
impl From<ProtocolStateMachine<InitGame>> for ProtocolStateMachine<InGame> {
    fn from(_: ProtocolStateMachine<InitGame>) -> ProtocolStateMachine<InGame> {
        ProtocolStateMachine {
            // Shared Values
            state: InGame {},
        }
    }
}

/// InGame will either end the current game or go to the next step (different modes: Step/RealTime)
// Transitions:
//      InGame -> Ended
impl From<ProtocolStateMachine<InGame>> for ProtocolStateMachine<Ended> {
    fn from(_: ProtocolStateMachine<InGame>) -> ProtocolStateMachine<Ended> {
        ProtocolStateMachine {
            // Shared Values
            state: Ended {},
        }
    }
}

/// InReplay will either end or go to the next step (different modes: Step/RealTime)
// Transitions
//      InReplay -> Ended
impl From<ProtocolStateMachine<InReplay>> for ProtocolStateMachine<Ended> {
    fn from(_: ProtocolStateMachine<InReplay>) -> ProtocolStateMachine<Ended> {
        ProtocolStateMachine {
            // Shared Values
            state: Ended {},
        }
    }
}

/// Once a game has ended we can either replay it, launch another one or end
// Transitions:
//      Ended -> Launched
//      Ended -> InGame
impl From<ProtocolStateMachine<Ended>> for ProtocolStateMachine<Launched> {
    fn from(_: ProtocolStateMachine<Ended>) -> ProtocolStateMachine<Launched> {
        ProtocolStateMachine {
            // Shared Values
            state: Launched {},
        }
    }
}

impl From<ProtocolStateMachine<Ended>> for ProtocolStateMachine<InGame> {
    fn from(_: ProtocolStateMachine<Ended>) -> ProtocolStateMachine<InGame> {
        ProtocolStateMachine {
            // Shared Values
            state: InGame {},
        }
    }
}
