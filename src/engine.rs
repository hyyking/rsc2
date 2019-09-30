use std::convert::TryInto;

use tokio::net::TcpStream;
use tokio::prelude::*;
use websocket::OwnedMessage;

use crate::states::*;
use rsc2_pb::prelude::*;
use rsc2_pb::sc2_api::{response, Observation, Response, ResponseObservation, Status};

fn play_to_completion(mut sm: ProtocolStateMachine<InGame>) -> ProtocolStateMachine<Ended> {
    let mut current_loop = 0;
    loop {
        let req_message = sm.inner.gamestate_request(current_loop).unwrap_or_quit();
        sm.shared = sm.shared.request_gamestate(req_message);
        match sm.shared.last_response.take() {
            Some(ret_message) => match ret_message {
                OwnedMessage::Binary(obs) => {
                    let observation = Response::decode(obs);
                    let Response {
                        status, response, ..
                    } = observation.unwrap();
                    if let Ok(Status::Ended) = status.unwrap().try_into() {
                        break;
                    }

                    if let response::Response::Observation(ResponseObservation {
                        observation,
                        ..
                    }) = response.unwrap()
                    {
                        let Observation { score, .. } = observation.unwrap();
                        trace!("{:?}", score.unwrap());
                    }
                }
                _ => trace!("Observed a non-binary buffer"),
            },

            None => trace!("Nothing observed"),
        }
        current_loop += 1;
    }
    sm.into()
}

pub struct ProtocolStateMachine<S>
where
    S: IsProtocolState,
{
    pub shared: SharedState,
    pub inner: S,
}

impl<S> ProtocolStateMachine<S>
where
    S: IsProtocolState,
{
    pub fn ping(&self) {
        debug!("pinged ProtocolStateMachine");
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum GameSpeed {
    RealTime = 0,
    Faster = 1,
    Step = 2,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ProtocolArg {
    CreateGame = 0,
    StartReplay = 1,
    JoinGame = 2,
    PlayGame = 3,
    RestartGame = 4,
    LeaveGame = 5,
}

pub enum ProtocolState {
    Launched(Option<ProtocolStateMachine<Launched>>),
    InitGame(ProtocolStateMachine<InitGame>),
    InGame(ProtocolStateMachine<InGame>),
    InReplay(ProtocolStateMachine<InReplay>),
    Ended(ProtocolStateMachine<Ended>),
    CloseGame,
}

impl ProtocolState {
    pub fn run(self, arg: ProtocolArg) -> ProtocolState {
        use self::{ProtocolArg::*, ProtocolState::*};
        match (self, arg) {
            (Launched(None), _) => CloseGame,
            (Launched(Some(mut sm)), CreateGame) => {
                let req = sm.inner.create_game_request(/* add context */);
                sm.shared = sm.shared.create_game(req.unwrap());
                InitGame(sm.into())
            }
            (Launched(Some(sm)), StartReplay) => {
                // sm.start_replay();
                InReplay(sm.into())
            }
            (Launched(Some(mut sm)), JoinGame) => {
                let req = sm.inner.join_game_request(/* add context */);
                sm.shared = sm.shared.join_game(req.unwrap_or_quit());
                InGame(sm.into())
            }
            (InitGame(mut sm), JoinGame) => {
                let req = sm.inner.join_game_request(/* add context */);
                sm.shared = sm.shared.join_game(req.unwrap_or_quit());
                InGame(sm.into())
            }
            (Ended(sm), RestartGame) => {
                // sm.restart_game();
                InGame(sm.into())
            }
            (InGame(sm), PlayGame) => Ended(play_to_completion(sm)),
            (Ended(_sm), LeaveGame) => {
                // sm.close_game();
                Launched(None)
            }
            (x, y) => {
                error!(
                    "Could not transition ProtocolState::{:?} with argument ProtocolArg::{:?}",
                    x, y
                );
                x
            }
        }
    }
}

impl std::fmt::Debug for ProtocolState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ProtocolState::*;
        match self {
            CloseGame => write!(f, "CloseGame"),
            Launched(_) => write!(f, "Launched"),
            InitGame(_) => write!(f, "InitGame"),
            InGame(_) => write!(f, "InGame"),
            InReplay(_) => write!(f, "InReplay"),
            Ended(_) => write!(f, "Ended"),
        }
    }
}

impl Into<ProtocolState> for websocket::client::r#async::ClientNew<TcpStream> {
    fn into(self) -> ProtocolState {
        ProtocolState::Launched(Some(ProtocolStateMachine {
            shared: SharedState {
                conn: self
                    .map_err(|e| error!("{:?}", e))
                    .wait()
                    .expect(r#"could not connect to the SC2API at "ws://127.0.0.1:5000/sc2api""#)
                    .0,
                last_response: None,
            },
            inner: Launched::default(),
        }))
    }
}

/// Once a game has ended we can either replay it, launch another one or end
// Transitions:
//      Ended -> Launched
//      Ended -> InGame
impl From<ProtocolStateMachine<Ended>> for ProtocolStateMachine<Launched> {
    fn from(prev: ProtocolStateMachine<Ended>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: Launched::default(),
        }
    }
}

impl From<ProtocolStateMachine<Ended>> for ProtocolStateMachine<InGame> {
    fn from(prev: ProtocolStateMachine<Ended>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InGame::default(),
        }
    }
}

/// InGame will either end the current game or go to the next step (different modes: Step/RealTime)
// Transitions:
//      InGame -> Ended
impl From<ProtocolStateMachine<InGame>> for ProtocolStateMachine<Ended> {
    fn from(prev: ProtocolStateMachine<InGame>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: Ended::default(),
        }
    }
}

/// InitGame will transition in InGame
// Transitions:
//      InitGame -> InGame
impl From<ProtocolStateMachine<InitGame>> for ProtocolStateMachine<InGame> {
    fn from(prev: ProtocolStateMachine<InitGame>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InGame::default(),
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
    fn from(prev: ProtocolStateMachine<Launched>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InitGame::default(),
        }
    }
}

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InGame> {
    fn from(prev: ProtocolStateMachine<Launched>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InGame::default(),
        }
    }
}

impl From<ProtocolStateMachine<Launched>> for ProtocolStateMachine<InReplay> {
    fn from(prev: ProtocolStateMachine<Launched>) -> Self {
        ProtocolStateMachine {
            shared: prev.shared,
            inner: InReplay::default(),
        }
    }
}

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
