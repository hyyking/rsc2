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

use websocket::sync::Client;

use tokio::net::TcpStream;
use tokio::reactor::Handle;

pub trait IsProtocolState {
    fn create_game(&self) {}
    fn join_game(&self) {}
    fn start_replay(&self) {}
    fn restart_game(&self) {}
    fn close_game(&self) {}
}

pub struct SharedState {
    pub conn: TcpStream,
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

    pub fn create_game(&self) {
        debug!("Creating game...");
        self.inner.create_game()
    }
    pub fn join_game(&self) {
        debug!("Joining game...");
        self.inner.join_game()
    }
    pub fn start_replay(&self) {
        debug!("Starting replay...");
        self.inner.start_replay()
    }
    pub fn restart_game(&self) {
        debug!("Restartin game...");
        self.inner.restart_game()
    }
    pub fn close_game(&self) {
        debug!("Closing game...");
        self.inner.close_game()
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

impl Into<ProtocolState> for Client<std::net::TcpStream> {
    fn into(self) -> ProtocolState {
        ProtocolState::Launched(Some(ProtocolStateMachine {
            shared: SharedState {
                conn: TcpStream::from_std(self.into_stream().0, &Handle::default())
                    .expect("couldn't convert std::net::TcpStream to tokio::net::TcpStream"),
            },
            inner: Launched::default(),
        }))
    }
}

impl ProtocolState {
    pub fn run(self, arg: ProtocolArg) -> ProtocolState {
        use self::{ProtocolArg::*, ProtocolState::*};
        match (self, arg) {
            (Launched(None), _) => CloseGame,
            (Launched(Some(sm)), CreateGame) => {
                sm.create_game();
                InitGame(sm.into())
            }
            (Launched(Some(sm)), StartReplay) => {
                sm.start_replay();
                InReplay(sm.into())
            }
            (Launched(Some(sm)), JoinGame) => {
                sm.join_game();
                InGame(sm.into())
            }
            (InitGame(sm), JoinGame) => {
                sm.join_game();
                InGame(sm.into())
            }
            (Ended(sm), RestartGame) => {
                sm.restart_game();
                InGame(sm.into())
            }
            (Ended(sm), LeaveGame) => {
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
