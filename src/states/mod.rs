pub mod states;

use states::*;
use websocket::sync::Client;

use tokio::net::TcpStream;
use tokio::reactor::Handle;

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
    Launched(ProtocolStateMachine<Launched>),
    InitGame(ProtocolStateMachine<InitGame>),
    InGame(ProtocolStateMachine<InGame>),
    InReplay(ProtocolStateMachine<InReplay>),
    Ended(ProtocolStateMachine<Ended>),
}

impl Into<ProtocolState> for Client<std::net::TcpStream> {
    fn into(self) -> ProtocolState {
        ProtocolState::Launched(ProtocolStateMachine {
            shared: SharedState {
                conn: TcpStream::from_std(self.into_stream().0, &Handle::default())
                    .expect("couldn't convert std::net::TcpStream to tokio::net::TcpStream"),
            },
            inner: Launched::default(),
        })
    }
}

impl ProtocolState {
    pub fn run(self, arg: ProtocolArg) -> ProtocolState {
        use self::{ProtocolArg::*, ProtocolState::*};
        match (self, arg) {
            (Launched(sm), CreateGame) => {
                sm.create_game();
                InitGame(sm.into())
            }
            (Launched(sm), StartReplay) => {
                sm.start_replay();
                InReplay(sm.into())
            }
            (Launched(sm), JoinGame) => {
                sm.join_game();
                InGame(m.into())
            }
            (InitGame(sm), JoinGame) => {
                sm.join_game();
                InGame(m.into())
            }
            (Ended(sm), RestartGame) => {
                sm.restart_game();
                InGame(m.into())
            }
            (Ended(sm), LeaveGame) => Launched(m.into()),
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
            Launched(_) => write!(f, "Launched"),
            InitGame(_) => write!(f, "InitGame"),
            InGame(_) => write!(f, "InGame"),
            InReplay(_) => write!(f, "InReplay"),
            Ended(_) => write!(f, "Ended"),
        }
    }
}
