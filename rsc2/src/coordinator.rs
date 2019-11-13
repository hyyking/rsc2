use crate::Commands;

use std::cell::Cell;
use std::convert::TryFrom;
use std::io;

use rsc2_pb::{codec, sc2_api};
use tokio::runtime::Runtime;

macro_rules! validate_status {
    ($status:expr => $variant:path) => {{
        let status: Option<i32> = $status;
        let _: sc2_api::Status = $variant;
        status
            .ok_or_else(|| io::Error::new(io::ErrorKind::ConnectionAborted, "Missing Status Code"))
            .and_then(|status| match sc2_api::Status::try_from(status).ok() {
                Some($variant) => Ok(()),
                Some(e) => Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    format!(r#"Unexpected "{:?}""#, e),
                )),
                None => Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Wrong status Code",
                )),
            })
    }};
}

macro_rules! bit_flag {
    ($set_f:ident as $val:literal; $check_f:ident) => {
        fn $check_f(&self) -> bool {
            let val: u32 = $val;
            self.0.get() & 2u8.pow(val) != 0
        }
        fn $set_f(&self) {
            let val: u32 = $val;
            let old = self.0.take();
            self.0.set(old | 2u8.pow(val));
        }
    };
}

// 0000 0000
// first bit: Launched ?
// second bit: InitGame ?
// third bit:  InReplay ?
// fourth bit: InGame ?
// fith bit: Ended ?
struct StateMachine(Cell<u8>);

impl StateMachine {
    fn new() -> Self {
        Self(Cell::new(0))
    }
    fn reset(&self) {
        drop(self.0.take());
    }
    bit_flag!(launched as 0; is_launched);
    bit_flag!(initgame as 1; is_initgame);
    bit_flag!(inreplay as 2; is_inreplay);
    bit_flag!(ingame as 3; is_ingame);
    bit_flag!(ended as 4; is_ended);
}

pub struct Coordinator {
    sm: StateMachine,
    conn: Option<codec::SC2ProtobufClient>,
}

impl Coordinator {
    pub fn new() -> Self {
        Self {
            sm: StateMachine::new(),
            conn: None,
        }
    }

    pub fn run<Iter, Item>(&mut self, elements: Iter) -> io::Result<u32>
    where
        Iter: IntoIterator<Item = Item>,
        Item: Into<Commands>,
    {
        let mut request_count: u32 = 0;
        let rt = tokio::runtime::Builder::new().build().unwrap();

        for element in elements.into_iter().map(|el| el.into()).into_iter() {
            request_count += 1;
            self.validate(&element)?;
            match element {
                Commands::Launched { .. } => {
                    let stream = rt.block_on(self.init_connection())?;
                    self.conn = Some(stream);
                    self.sm.launched()
                }
                Commands::CreateGame { request } => {
                    let request = sc2_api::Request::with_id(request, request_count);
                    let response =
                        rt.block_on(requests::with_response(self.get_mut_stream()?, request))?;
                    validate_status!(response.status => sc2_api::Status::InitGame)?;

                    self.sm.initgame()
                }
                Commands::JoinGame { request } => {
                    let request = sc2_api::Request::with_id(request, request_count);
                    let response =
                        rt.block_on(requests::with_response(self.get_mut_stream()?, request))?;
                    validate_status!(response.status => sc2_api::Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::StartReplay { request } => {
                    let request = sc2_api::Request::with_id(request, request_count);
                    let response =
                        rt.block_on(requests::with_response(self.get_mut_stream()?, request))?;
                    validate_status!(response.status => sc2_api::Status::InReplay)?;

                    self.sm.inreplay();
                    // Execute game logic
                    self.sm.ended();
                }
                Commands::RestartGame { .. } => {
                    let request =
                        sc2_api::Request::with_id(sc2_api::RequestRestartGame {}, request_count);
                    let response =
                        rt.block_on(requests::with_response(self.get_mut_stream()?, request))?;
                    validate_status!(response.status => sc2_api::Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::LeaveGame { .. } => {
                    let request =
                        sc2_api::Request::with_id(sc2_api::RequestLeaveGame {}, request_count);
                    let response =
                        rt.block_on(requests::with_response(self.get_mut_stream()?, request))?;
                    validate_status!(response.status => sc2_api::Status::Launched)?;

                    self.sm.reset();
                    self.sm.launched();
                }
                Commands::QuitGame { .. } => {
                    let request = sc2_api::Request::with_id(sc2_api::RequestQuit {}, request_count);
                    let response =
                        rt.block_on(requests::with_response(self.get_mut_stream()?, request))?;
                    validate_status!(response.status => sc2_api::Status::Quit)?;

                    self.sm.reset();
                }
            }
        }
        Ok(request_count)
    }

    fn validate(&self, command: &Commands) -> io::Result<()> {
        let fast_err = |message: &'static str| -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::InvalidInput, message))
        };
        let is_launched = || -> io::Result<()> {
            if self.sm.is_launched() {
                return Ok(());
            }
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Game has not been launched",
            ))
        };
        match command {
            Commands::Launched { .. } => {
                if self.sm.is_launched() {
                    return fast_err("Game is already launched");
                }
            }
            Commands::CreateGame { .. } => {
                is_launched()?;
                if self.sm.is_ingame() || self.sm.is_inreplay() || self.sm.is_initgame() {
                    return fast_err("Cannot create a game while one is running");
                }
            }
            Commands::JoinGame { .. } => {
                is_launched()?;
                if self.sm.is_ingame() || self.sm.is_inreplay() {
                    return fast_err("Cannot join a game while one is running");
                }
            }
            Commands::StartReplay { .. } => {
                is_launched()?;
                if self.sm.is_initgame() || self.sm.is_ingame() {
                    return fast_err("Cannot play a replay while a game is running");
                }
            }
            Commands::RestartGame { .. } => {
                if !self.sm.is_ended() || !self.sm.is_ingame() {
                    return fast_err("Cannot restart a game while one is running");
                }
            }
            Commands::LeaveGame { .. } => {
                if !self.sm.is_ended() {
                    return fast_err("Cannot leave a game while one is running");
                }
            }
            Commands::QuitGame { .. } => {}
        }
        Ok(())
    }

    async fn init_connection(&self) -> io::Result<codec::SC2ProtobufClient> {
        use websocket_lite::ClientBuilder;
        let builder = ClientBuilder::new("ws://127.0.0.1:5000/sc2api").unwrap();
        Ok(codec::from_framed(
            builder
                .async_connect_insecure()
                .await
                .map_err(|e| *e.downcast::<io::Error>().unwrap())?,
        ))
    }

    fn get_mut_stream(&mut self) -> io::Result<&mut codec::SC2ProtobufClient> {
        self.conn.as_mut().ok_or(io::Error::new(
            io::ErrorKind::NotConnected,
            "Engine is not connected to a SC2 instance",
        ))
    }

    fn play_game(&mut self, rt: &Runtime, start: u32) -> io::Result<u32> {
        //use rsc2_pb::prelude::*;
        use sc2_api::{
            response::Response::Observation as Obs, Request as Req, RequestObservation as ReqObs,
            ResponseObservation as RespObs, Status,
        };

        let mut count = start;
        let conn = self.get_mut_stream()?;
        loop {
            count += 1;
            let request = Req::with_id(ReqObs::nofog(count), count);
            let response = rt.block_on(requests::with_filter_response(conn, request))?;
            validate_status!(response.status => Status::InGame)?;
            match response {
                sc2_api::Response {
                    status,
                    response: Some(response),
                    ..
                } => {
                    if validate_status!(status => Status::Ended).is_ok() {
                        break Ok(count);
                    }
                    if let Obs(RespObs { observation: _, .. }) = response {}
                }
                r @ _ => eprintln!("unhandled response:\n{:#?}", r),
            }
        }
    }
}

mod requests {
    use std::io;

    use futures::{
        future::{ready, Ready},
        sink::SinkExt,
        stream::{Stream, StreamExt},
    };
    use rsc2_pb::{codec::SC2ProtobufClient, sc2_api};

    pub(crate) async fn with_response(
        conn: &mut SC2ProtobufClient,
        request: sc2_api::Request,
    ) -> io::Result<sc2_api::Response> {
        conn.send(request).await?;
        conn.next().await.ok_or(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "No response for request",
        ))?
    }

    pub(crate) async fn with_filter_response(
        conn: &mut SC2ProtobufClient,
        request: sc2_api::Request,
    ) -> io::Result<sc2_api::Response> {
        let id = request.id;
        let mut conn = conn.filter(|resp| id_filter(resp, id));
        conn.send(request).await?;
        conn.next().await.ok_or(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "No response for request",
        ))?
    }
    fn id_filter(resp: &<SC2ProtobufClient as Stream>::Item, id: Option<u32>) -> Ready<bool> {
        ready(resp.as_ref().ok().filter(|r| r.id == id).is_some())
    }
}

#[cfg(test)]
mod coordinator_states {
    use super::*;
    use rsc2_pb::{prelude::*, sc2_api};
    use std::io;
    use Commands::*;

    #[allow(non_snake_case)]
    fn REQUESTJOINGAME() -> sc2_api::RequestJoinGame {
        sc2_api::RequestJoinGame::default_config()
    }
    #[allow(non_snake_case)]
    fn REQUESTCREATEGAME() -> sc2_api::RequestCreateGame {
        sc2_api::RequestCreateGame::default_config()
    }
    #[allow(non_snake_case)]
    fn REQUESTSTARTREPLAY() -> sc2_api::RequestStartReplay {
        sc2_api::RequestStartReplay::from_file("")
    }

    impl Coordinator {
        fn mock_run<Iter, Item>(&mut self, elements: Iter) -> io::Result<()>
        where
            Iter: IntoIterator<Item = Item>,
            Item: Into<Commands>,
        {
            for element in elements.into_iter().map(|el| el.into()).into_iter() {
                self.validate(&element)?;
                match element {
                    Commands::Launched { .. } => self.sm.launched(),
                    Commands::CreateGame { .. } => self.sm.initgame(),
                    Commands::JoinGame { .. } => {
                        self.sm.ingame();
                        self.sm.ended();
                    }
                    Commands::StartReplay { .. } => {
                        self.sm.inreplay();
                        self.sm.ended();
                    }
                    Commands::RestartGame { .. } => {
                        self.sm.ingame();
                        self.sm.ended();
                    }
                    Commands::LeaveGame { .. } => {
                        self.sm.reset();
                        self.sm.launched();
                    }
                    Commands::QuitGame { .. } => self.sm.reset(),
                }
            }
            Ok(())
        }
    }

    #[test]
    fn create() -> io::Result<()> {
        let mut c = Coordinator::new();
        c.sm.launched(); // Mock the launching of the game
        c.mock_run(&[
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
            JoinGame {
                request: REQUESTJOINGAME(),
            },
            LeaveGame {},
        ])
    }

    #[test]
    fn two_games() -> io::Result<()> {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
            JoinGame {
                request: REQUESTJOINGAME(),
            },
            LeaveGame {},
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
            JoinGame {
                request: REQUESTJOINGAME(),
            },
            RestartGame {},
            LeaveGame {},
        ])
    }

    #[test]
    fn join() -> io::Result<()> {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            JoinGame {
                request: REQUESTJOINGAME(),
            },
            LeaveGame {},
        ])
    }

    #[test]
    fn restart() -> io::Result<()> {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
            JoinGame {
                request: REQUESTJOINGAME(),
            },
            RestartGame {},
        ])
    }

    #[test]
    fn replay() -> io::Result<()> {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            StartReplay {
                request: REQUESTSTARTREPLAY(),
            },
            LeaveGame {},
        ])
    }

    #[test]
    #[should_panic]
    fn create_twice() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
        ])
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn join_then_create() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            JoinGame {
                request: REQUESTJOINGAME(),
            },
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
        ])
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn create_and_replay_twice() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
            StartReplay {
                request: REQUESTSTARTREPLAY(),
            },
        ])
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn restart_after_quit() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            CreateGame {
                request: REQUESTCREATEGAME(),
            },
            JoinGame {
                request: REQUESTJOINGAME(),
            },
            LeaveGame {},
            RestartGame {},
        ])
        .unwrap()
    }
    #[test]
    #[should_panic]
    fn replay_restart() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.mock_run(&[
            StartReplay {
                request: REQUESTSTARTREPLAY(),
            },
            RestartGame {},
        ])
        .unwrap()
    }
}
