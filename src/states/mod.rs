use tokio::prelude::*;
use tokio::{codec::Framed, net::TcpStream};
use websocket::{r#async::MessageCodec, OwnedMessage};

use rsc2_pb::prelude::*;

mod ended;
use self::ended::Ended;

mod ingame;
use self::ingame::InGame;

mod init;
use self::init::InitGame;

mod launch;
use self::launch::Launched;

mod replay;
use self::replay::InReplay;

pub struct SharedState {
    pub conn: Framed<TcpStream, MessageCodec<OwnedMessage>>,
    pub last_response: Option<OwnedMessage>,
}

impl SharedState {
    /// Synchronous messages because next state depends on this
    pub fn create_game(self, message: OwnedMessage) -> Self {
        info!("Creating game...");
        let conn = self
            .conn
            .send(message)
            .wait()
            .expect("Couldn't send 'create_game' query");
        let (response, stream) = conn
            .into_future()
            .map_err(|err| error!("{:?}", err.0))
            .wait()
            .expect("Couldn't wait for 'create_game' response");
        debug!("{:?}", &response);
        Self {
            conn: stream,
            last_response: Some(response.unwrap()),
            ..self
        }
    }
    pub fn join_game(self, message: OwnedMessage) -> Self {
        info!("Joining game...");
        let conn = self
            .conn
            .send(message)
            .wait()
            .expect("Couldn't send 'join_game' query");
        let (response, stream) = conn
            .into_future()
            .map_err(|err| error!("{:?}", err.0))
            .wait()
            .expect("Couldn't wait for 'join_game' response");
        debug!("{:?}", &response);
        Self {
            conn: stream,
            last_response: Some(response.unwrap()),
            ..self
        }
    }
    pub fn start_replay(self) -> Self {
        debug!("Starting replay...");
        Self { ..self }
    }
    pub fn restart_game(self) -> Self {
        debug!("Restartin game...");
        Self { ..self }
    }
    pub fn close_game(self) -> Self {
        debug!("Closing game...");
        Self { ..self }
    }
}

pub trait IsProtocolState: std::fmt::Debug {
    fn create_game_request(&self) -> EncodeResult {
        error!("{:?}: cannot create 'create_game' request", self);
        panic!("Invalid Operation");
    }
    fn join_game_request(&self) -> EncodeResult {
        error!("{:?}: cannot create 'join_game' request", self);
        panic!("Invalid Operation");
    }
    fn start_replay_request(&self) {
        error!("{:?}: cannot create 'join_game' request", self);
        panic!("Invalid Operation");
    }
    fn restart_game_request(&self) {
        error!("{:?}: cannot create 'restart_game' request", self);
        panic!("Invalid Operation");
    }
    fn close_game_request(&self) {
        error!("{:?}: cannot create 'close_game' request", self);
        panic!("Invalid Operation");
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

impl Into<ProtocolState> for websocket::client::r#async::ClientNew<websocket::r#async::TcpStream> {
    fn into(self) -> ProtocolState {
        ProtocolState::Launched(Some(ProtocolStateMachine {
            shared: SharedState {
                conn: self
                    .map(|(s, _)| s)
                    .wait()
                    .expect(r#"could not connect to the SC2API at "ws://127.0.0.1:5000/sc2api""#),
                last_response: None,
            },
            inner: Launched::default(),
        }))
    }
}
