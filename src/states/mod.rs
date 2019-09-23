mod ended;
mod ingame;
mod init;
mod launch;
mod replay;

use ended::Ended;
use ingame::InGame;
use init::InitGame;
use launch::Launched;
use replay::InReplay;

use tokio::prelude::*;

pub struct SharedState {
    pub conn: tokio::codec::Framed<tokio::net::TcpStream, websocket::r#async::MessageCodec<websocket::OwnedMessage>>
}

pub trait IsProtocolState {
    fn create_game(&mut self, shared: &mut SharedState) {
        error!("Wrong Caller of create_game");
    }
    fn join_game(&mut self, _: &mut SharedState) {
        error!("Wrong Caller of join_game")
    }
    fn start_replay(&mut self, _: &mut SharedState) {
        error!("Wrong Caller of start_replay")
    }
    fn restart_game(&mut self, _: &mut SharedState) {
        error!("Wrong Caller of restart_game")
    }
    fn close_game(&mut self, _: &mut SharedState) {
        error!("Wrong Caller of close_game")
    }
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

    pub fn create_game(&mut self) {
        debug!("Creating game...");
        self.inner.create_game(&mut self.shared);
    }
    pub fn join_game(&mut self) {
        debug!("Joining game...");
        self.inner.join_game(&mut self.shared);
    }
    pub fn start_replay(&mut self) {
        debug!("Starting replay...");
        self.inner.start_replay(&mut self.shared);
    }
    pub fn restart_game(&mut self) {
        debug!("Restartin game...");
        self.inner.restart_game(&mut self.shared);
    }
    pub fn close_game(&mut self) {
        debug!("Closing game...");
        self.inner.close_game(&mut self.shared);
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum ProtocolArg {
    CreateGame = 0,
    StartReplay,
    JoinGame,
    Step,
    RestartGame,
    LeaveGame,
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
                sm.create_game();
                InitGame(sm.into())
            }
            (Launched(Some(mut sm)), StartReplay) => {
                sm.start_replay();
                InReplay(sm.into())
            }
            (Launched(Some(mut sm)), JoinGame) => {
                sm.join_game();
                InGame(sm.into())
            }
            (InitGame(mut sm), JoinGame) => {
                sm.join_game();
                InGame(sm.into())
            }
            (Ended(mut sm), RestartGame) => {
                sm.restart_game();
                InGame(sm.into())
            }
            (Ended(mut sm), LeaveGame) => {
                sm.close_game();
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

impl Into<ProtocolState> for websocket::client::r#async::ClientNew<websocket::r#async::TcpStream> {
    fn into(self) -> ProtocolState {
        ProtocolState::Launched(Some(ProtocolStateMachine {
            shared: SharedState {
                conn: self
                    .map(|(s, _)| s)
                    .wait()
                    .expect(r#"could not connect to the SC2API at "ws://127.0.0.1:5000/sc2api""#),
            },
            inner: Launched::default(),
        }))
    }
}
