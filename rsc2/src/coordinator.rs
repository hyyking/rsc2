use std::cell::Cell;
use std::io;
use std::sync::mpsc;

use crate::agent;
use crate::Commands;

use futures::stream::StreamExt;
use rsc2_pb::{
    codec::SC2ProtobufClient,
    sc2_api::{
        Request, RequestLeaveGame, RequestObservation, RequestQuit, RequestRestartGame, Response,
        Status,
    },
    validate_status,
};
use tokio::{
    runtime::Runtime,
    stream::iter,
    sync::{oneshot, watch},
};

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

pub struct Coordinator<T>
where
    T: agent::AgentHook + 'static,
{
    sm: StateMachine,
    conn: Cell<Option<SC2ProtobufClient>>,
    agent: Cell<Option<T>>,
}

impl<T> Coordinator<T>
where
    T: agent::AgentHook,
{
    pub fn new() -> Self {
        Self {
            sm: StateMachine::new(),
            conn: Cell::new(None),
            agent: Cell::new(None),
        }
    }

    pub fn run<Iter, Item>(&self, elements: Iter) -> io::Result<u32>
    where
        Item: Into<Commands<T>>,
        Iter: IntoIterator<Item = Item>,
    {
        let mut request_count: u32 = 0;
        let rt = tokio::runtime::Builder::new()
            .core_threads(4)
            .build()
            .unwrap();

        for element in elements.into_iter().map(|el| el.into()).into_iter() {
            request_count += 1;
            self.validate(&element)?;
            match element {
                Commands::Launched { socket, .. } => {
                    self.conn
                        .set(Some(rt.block_on(requests::init_connection(socket))?));
                    self.sm.launched()
                }
                Commands::CreateGame { request } => {
                    let response =
                        self.send_request(&rt, Request::with_id(request, request_count))?;
                    validate_status!(response.status => Status::InitGame)?;

                    self.sm.initgame()
                }
                Commands::JoinGame { request, agent } => {
                    self.agent.set(Some(agent));

                    let response =
                        self.send_request(&rt, Request::with_id(request, request_count))?;
                    validate_status!(response.status => Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::StartReplay { request, agent } => {
                    self.agent.set(Some(agent));

                    let response =
                        self.send_request(&rt, Request::with_id(request, request_count))?;
                    validate_status!(response.status => Status::InReplay)?;

                    self.sm.inreplay();
                    // Execute game logic
                    self.sm.ended();
                }
                Commands::RestartGame { .. } => {
                    let response = self.send_request(
                        &rt,
                        Request::with_id(RequestRestartGame {}, request_count),
                    )?;
                    validate_status!(response.status => Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::LeaveGame { .. } => {
                    let response = self
                        .send_request(&rt, Request::with_id(RequestLeaveGame {}, request_count))?;
                    validate_status!(response.status => Status::Launched)?;

                    self.sm.reset();
                    self.sm.launched();
                }
                Commands::QuitGame { .. } => {
                    let response =
                        self.send_request(&rt, Request::with_id(RequestQuit {}, request_count))?;
                    validate_status!(response.status => Status::Quit)?;

                    self.sm.reset();
                }
            }
        }
        rt.shutdown_now();
        Ok(request_count)
    }
    /// {{{
    fn validate(&self, command: &Commands<T>) -> io::Result<()> {
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
    /// }}}

    fn get_stream(&self) -> io::Result<SC2ProtobufClient> {
        self.conn.take().ok_or(io::Error::new(
            io::ErrorKind::NotConnected,
            "Engine is not connected to a SC2 instance",
        ))
    }

    fn get_agent(&self) -> io::Result<T> {
        self.agent.take().ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Engine has no agent to run",
        ))
    }

    fn send_request(&self, rt: &Runtime, request: Request) -> io::Result<Response> {
        let mut conn = self.get_stream()?;
        let response = rt.block_on(requests::filter_response(&mut conn, request))?;
        self.conn.set(Some(conn));
        Ok(response)
    }

    fn play_game(&self, rt: &Runtime, mut count: u32) -> io::Result<u32> {
        let mut conn = self.get_stream()?;
        let agent = self.get_agent()?;

        let game_result: io::Result<u32> = rt.block_on(async {
            let (mut broadcaster, mut watcher) = watch::channel(MachineStatus::Empty);
            let (req_producer, req_receiver) = mpsc::channel();
            let (ret_agent, rec_agent) = oneshot::channel();

            // Agent side
            rt.spawn(requests::agent_client(
                agent,
                req_producer.clone(),
                watcher.clone(),
                ret_agent,
            ));

            // Observation requests producer
            rt.spawn(async move {
                while let Some(_) = watcher.recv_ref().await {
                    let req = RequestObservation::nofog(count); // Make better observations
                    if req_producer.send(req.into()).is_err() {
                        break;
                    };
                }
            });

            let mut reqstream = iter(req_receiver.try_iter());
            loop {
                let request = match reqstream.next().await {
                    Some(req) => req,
                    None => continue,
                };

                count += 1;
                let r =
                    requests::filter_response(&mut conn, Request::with_id(request, count)).await;
                if r.is_err() {
                    broadcaster.closed().await;
                };
                let response = r?;

                if validate_status!(response.status => Status::Ended).is_ok() {
                    broadcaster.closed().await;
                    break;
                };
                broadcaster
                    .broadcast(MachineStatus::Full(response))
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::BrokenPipe, "responses channel closed")
                    })?;
            }
            self.agent.set(rec_agent.await.ok());
            Ok(count)
        });
        self.conn.set(Some(conn));
        game_result
    }
}

pub(self) enum MachineStatus {
    Empty,
    Full(Response),
}

mod requests {
    use std::io;
    use std::net::SocketAddrV4;
    use std::sync::mpsc;

    use super::MachineStatus;
    use crate::agent::AgentHook;

    use futures::{
        future::{ready, Ready},
        sink::SinkExt,
        stream::{Stream, StreamExt},
    };
    use rsc2_pb::{
        codec::{from_framed, SC2ProtobufClient},
        sc2_api::{request::Request as rRequest, Request, Response},
    };
    use tokio::sync::{oneshot, watch};
    use websocket_lite::ClientBuilder;

    pub(super) async fn init_connection(addr: SocketAddrV4) -> io::Result<SC2ProtobufClient> {
        let builder =
            ClientBuilder::new(&format!("ws://{}:{}/sc2api", addr.ip(), addr.port())).unwrap();
        Ok(from_framed(
            builder
                .async_connect_insecure()
                .await
                .map_err(|e| *e.downcast::<io::Error>().unwrap())?,
        ))
    }

    pub(super) async fn filter_response(
        conn: &mut SC2ProtobufClient,
        request: Request,
    ) -> io::Result<Response> {
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

    pub(super) async fn agent_client<T>(
        mut agent: T,
        client: mpsc::Sender<rRequest>,
        mut value: watch::Receiver<MachineStatus>,
        retagent: oneshot::Sender<T>,
    ) where
        T: AgentHook,
    {
        if let Some(request) = agent.on_start_hook() {
            if client.send(request.into()).is_err() {
                let _ = retagent.send(agent);
                return;
            }
        }
        while let Some(watched) = value.recv_ref().await {
            let response = match *watched {
                MachineStatus::Full(ref resp) => resp,
                _ => continue,
            };
            if let Some(request) = agent.on_step_hook(response) {
                if client.send(request.into()).is_err() {
                    break;
                }
            }
        }
        agent.on_close_hook();
        let _ = retagent.send(agent);
    }
}
