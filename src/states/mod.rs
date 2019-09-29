use tokio::prelude::*;
use tokio::{codec::Framed, net::TcpStream};
use websocket::OwnedMessage;

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

type FramedStream = Framed<TcpStream, websocket::r#async::MessageCodec<OwnedMessage>>;

fn send_and_receive_sync(
    conn: FramedStream,
    message: OwnedMessage,
) -> (Option<OwnedMessage>, FramedStream) {
    conn.send(message)
        .map_err(|err| error!("Error Sending Message: {:?}", err))
        .map(|stream| {
            stream
                .into_future()
                .map_err(|err| error!("Error Waiting for Response: {:?}", err.0))
                .wait() // wait for the reponse
                .expect("Couldn't resolve response")
        })
        .wait()
        .expect("Couldn't resolve 'send_and_receive_sync' future")
}

pub struct SharedState {
    pub conn: FramedStream,
    pub last_response: Option<OwnedMessage>,
}

impl SharedState {
    /// Synchronous messages because next state depends on this
    pub fn create_game(self, message: OwnedMessage) -> Self {
        info!("Creating game...");
        let (response, stream): (Option<OwnedMessage>, FramedStream) =
            send_and_receive_sync(self.conn, message);
        debug!("{:?}", &response);
        Self {
            conn: stream,
            last_response: response,
            ..self
        }
    }
    pub fn join_game(self, message: OwnedMessage) -> Self {
        info!("Joining game...");
        let (response, stream): (Option<OwnedMessage>, FramedStream) =
            send_and_receive_sync(self.conn, message);
        debug!("{:?}", &response);
        Self {
            conn: stream,
            last_response: response,
            ..self
        }
    }
    pub fn request_gamestate(self, message: OwnedMessage) -> Self {
        let (response, stream): (Option<OwnedMessage>, FramedStream) =
            send_and_receive_sync(self.conn, message);
        Self {
            conn: stream,
            last_response: response,
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
        error!("{:?}: cannot create 'start_replay_request' request", self);
        panic!("Invalid Operation");
    }
    fn gamestate_request(&self, _: u32) -> EncodeResult {
        error!("{:?}: cannot create 'gamestate_request' request", self);
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

fn play_to_completion(mut sm: ProtocolStateMachine<InGame>) -> ProtocolStateMachine<InGame> {
    use rsc2_pb::sc2_api::{response, Observation, Response, ResponseObservation, Status};
    use std::convert::TryInto;
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
    sm
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
            (InGame(sm), PlayGame) => Ended(play_to_completion(sm).into()),
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
