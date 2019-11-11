pub mod agent;

mod commands;
pub use commands::*;

use std::cell::Cell;
use std::convert::TryFrom;
use std::io;

use prost::Message;
use rsc2_pb::{prelude::*, sc2_api};
use tokio::net::TcpStream;
use tokio::prelude::*;
use websocket_lite::AsyncClient;

macro_rules! validate_status {
    ($status:expr => $variant:path) => {
        let status: Option<i32> = $status;
        let _: sc2_api::Status = $variant;
        status
            .ok_or_else(|| io::Error::new(io::ErrorKind::ConnectionAborted, "Missing Status Code"))
            .and_then(|status| match sc2_api::Status::try_from(status).ok() {
                Some($variant) => Ok(()),
                Some(_) | None => Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Wrong status Code",
                )),
            })?;
    };
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
    conn: Option<AsyncClient<TcpStream>>,
}

impl Coordinator {
    pub fn new() -> Self {
        Self {
            sm: StateMachine::new(),
            conn: None,
        }
    }

    pub fn run<Iter, Item>(&mut self, elements: Iter) -> io::Result<()>
    where
        Iter: IntoIterator<Item = Item>,
        Item: Into<Commands>,
    {
        let rt = tokio::runtime::Builder::new().build().unwrap();

        for element in elements.into_iter().map(|el| el.into()).into_iter() {
            self.validate(&element)?;
            match element {
                Commands::Launched { .. } => {
                    let stream = rt.block_on(self.init_connection())?;
                    self.conn = Some(stream);
                    self.sm.launched()
                }
                Commands::CreateGame { .. } => {
                    let response = rt.block_on(self.create_game())?;
                    validate_status!(response.status => sc2_api::Status::InitGame);
                    self.sm.initgame()
                }
                Commands::JoinGame { .. } => {
                    let response = rt.block_on(self.join_game())?;
                    validate_status!(response.status => sc2_api::Status::InGame);
                    self.sm.ingame();
                    // Execute game logic
                    self.sm.ended();
                }
                Commands::StartReplay { .. } => {
                    self.sm.inreplay();
                    // Execute replay logic
                    self.sm.ended();
                }
                Commands::RestartGame { .. } => {
                    self.sm.ingame();
                    // Execute game logic
                    self.sm.ended();
                }
                Commands::LeaveGame { .. } => {
                    self.sm.reset();
                    self.sm.launched();
                }
            }
        }
        Ok(())
    }

    fn validate(&self, command: &Commands) -> io::Result<()> {
        let fast_err = |message: &'static str| -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::InvalidInput, message))
        };
        match command {
            Commands::Launched { .. } => {
                if self.sm.is_launched() {
                    return fast_err("Game is already launched");
                }
            }
            Commands::CreateGame { .. } => {
                self.is_launched()?;
                if self.sm.is_ingame() || self.sm.is_inreplay() || self.sm.is_initgame() {
                    return fast_err("Cannot create a game while one is running");
                }
            }
            Commands::JoinGame { .. } => {
                self.is_launched()?;
                if self.sm.is_ingame() || self.sm.is_inreplay() {
                    return fast_err("Cannot join a game while one is running");
                }
            }
            Commands::StartReplay { .. } => {
                self.is_launched()?;
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
        }
        Ok(())
    }

    fn get_mut_stream(&mut self) -> io::Result<&mut AsyncClient<TcpStream>> {
        self.conn.as_mut().ok_or(io::Error::new(
            io::ErrorKind::NotConnected,
            "Engine is not connected to a SC2 instance",
        ))
    }

    fn is_launched(&self) -> io::Result<()> {
        if !self.sm.is_launched() {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Game has not been launched",
            ))
        } else {
            Ok(())
        }
    }
}

impl Coordinator {
    async fn init_connection(&mut self) -> io::Result<AsyncClient<TcpStream>> {
        let builder = websocket_lite::ClientBuilder::new("ws://127.0.0.1:5000/sc2api").unwrap();
        builder
            .async_connect_insecure()
            .await
            .map_err(|e| *e.downcast::<io::Error>().unwrap())
    }

    async fn join_game(&mut self) -> io::Result<sc2_api::Response> {
        let conn = self.get_mut_stream()?;
        let request: EncodeResult = sc2_api::Request::with_id(
            sc2_api::RequestJoinGame::with_race(sc2_api::Race::Terran),
            1,
        )
        .into();

        conn.send(request.unwrap())
            .await
            .map_err(|e| *e.downcast::<io::Error>().unwrap())?;

        let response = conn
            .next()
            .await
            .ok_or(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "No response for create_game",
            ))?
            .map_err(|e| *e.downcast::<io::Error>().unwrap())?;

        Ok(sc2_api::Response::decode(response.into_data()).unwrap())
    }

    async fn create_game(&mut self) -> io::Result<sc2_api::Response> {
        let conn = self.get_mut_stream()?;
        let request: EncodeResult =
            sc2_api::Request::with_id(sc2_api::RequestCreateGame::default_config(), 0).into();

        conn.send(request.unwrap())
            .await
            .map_err(|e| *e.downcast::<io::Error>().unwrap())?;

        let response = conn
            .next()
            .await
            .ok_or(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "No response for create_game",
            ))?
            .map_err(|e| *e.downcast::<io::Error>().unwrap())?;

        Ok(sc2_api::Response::decode(response.into_data()).unwrap())
    }
}

#[cfg(test)]
mod coordinator_states {
    use super::*;
    use Commands::*;

    impl Coordinator {
        // Validate all the states without running them
        fn validate_all<Iter, Item>(&mut self, elements: Iter) -> io::Result<()>
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
                }
            }
            Ok(())
        }
    }

    #[test]
    fn create() {
        let mut c = Coordinator::new();
        c.sm.launched(); // Mock the launching of the game
        c.validate_all(&[CreateGame {}, JoinGame {}, LeaveGame {}])
            .unwrap();
    }

    #[test]
    fn two_games() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[
            CreateGame {},
            JoinGame {},
            LeaveGame {},
            CreateGame {},
            JoinGame {},
            RestartGame {},
            LeaveGame {},
        ])
        .unwrap()
    }

    #[test]
    fn join() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[JoinGame {}, LeaveGame {}]).unwrap()
    }

    #[test]
    fn restart() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[CreateGame {}, JoinGame {}, RestartGame {}])
            .unwrap()
    }

    #[test]
    fn replay() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[StartReplay {}, LeaveGame {}]).unwrap()
    }

    #[test]
    #[should_panic]
    fn create_twice() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[CreateGame {}, CreateGame {}]).unwrap()
    }

    #[test]
    #[should_panic]
    fn join_then_create() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[JoinGame {}, CreateGame {}]).unwrap()
    }

    #[test]
    #[should_panic]
    fn create_and_replay_twice() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[CreateGame {}, StartReplay {}]).unwrap()
    }

    #[test]
    #[should_panic]
    fn restart_after_quit() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[CreateGame {}, JoinGame {}, LeaveGame {}, RestartGame {}])
            .unwrap()
    }
    #[test]
    #[should_panic]
    fn replay_restart() {
        let mut c = Coordinator::new();
        c.sm.launched();
        c.validate_all(&[StartReplay {}, RestartGame {}]).unwrap()
    }
}
