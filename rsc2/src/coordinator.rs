use std::cell::Cell;
use std::io;

use crate::agent;
use crate::producer;
use crate::Commands;

use futures::stream::StreamExt;
use rsc2_pb::{
    codec::SC2ProtobufClient,
    sc2_api::{
        request::Request as rRequest, Request, RequestLeaveGame, RequestQuit, RequestRestartGame,
        Response, Status,
    },
    validate_status,
};
use tokio::runtime::Runtime;

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

pub struct Coordinator<A, P>
where
    A: agent::AgentHook + 'static,
    P: Iterator<Item = rRequest> + Unpin,
{
    sm: StateMachine,
    conn: Cell<Option<SC2ProtobufClient>>,
    agent: Cell<Option<A>>,
    producer: Cell<Option<P>>,
}

impl<A, P> Coordinator<A, P>
where
    A: agent::AgentHook + 'static,
    P: Iterator<Item = rRequest> + Unpin,
{
    pub fn new() -> Self {
        Self {
            sm: StateMachine::new(),
            conn: Cell::new(None),
            agent: Cell::new(None),
            producer: Cell::new(None),
        }
    }

    pub fn run<Iter, Item>(&self, elements: Iter) -> io::Result<u32>
    where
        Item: Into<Commands<A, P>>,
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
                    log::debug!("Commands::Launched");
                    self.conn
                        .set(Some(rt.block_on(requests::init_connection(socket))?));
                    self.sm.launched()
                }
                Commands::CreateGame { request } => {
                    log::debug!("Commands::CreateGame");
                    let response =
                        self.send_request(&rt, Request::with_id(request, request_count))?;
                    validate_status!(response.status => Status::InitGame)?;

                    self.sm.initgame()
                }
                Commands::JoinGame {
                    request,
                    agent,
                    producer,
                } => {
                    log::debug!("Commands::JoinGame");
                    self.agent.set(Some(agent));
                    self.producer.set(Some(producer));

                    let response =
                        self.send_request(&rt, Request::with_id(request, request_count))?;
                    validate_status!(response.status => Status::InGame)?;

                    self.sm.ingame();
                    self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::StartReplay {
                    request,
                    agent,
                    producer,
                } => {
                    log::debug!("Commands::StartReplay");
                    self.agent.set(Some(agent));
                    self.producer.set(Some(producer));

                    let response =
                        self.send_request(&rt, Request::with_id(request, request_count))?;
                    validate_status!(response.status => Status::InReplay)?;

                    self.sm.inreplay();
                    self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::RestartGame { .. } => {
                    log::debug!("Commands::RestartGame");
                    let response = self.send_request(
                        &rt,
                        Request::with_id(RequestRestartGame {}, request_count),
                    )?;
                    validate_status!(response.status => Status::InGame)?;

                    self.sm.ingame();
                    self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::LeaveGame { .. } => {
                    log::debug!("Commands::LeaveGame");
                    let response = self
                        .send_request(&rt, Request::with_id(RequestLeaveGame {}, request_count))?;
                    validate_status!(response.status => Status::Launched)?;

                    self.sm.reset();
                    self.sm.launched();
                }
                Commands::QuitGame { .. } => {
                    log::debug!("Commands::QuitGame");
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
    fn validate(&self, command: &Commands<A, P>) -> io::Result<()> {
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

    fn get_agent(&self) -> io::Result<A> {
        self.agent.take().ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Engine has no agent to run",
        ))
    }

    fn get_producer(&self) -> io::Result<P> {
        self.producer.take().ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Engine has no producer to run",
        ))
    }

    fn send_request(&self, rt: &Runtime, request: Request) -> io::Result<Response> {
        log::trace!("send_request");
        let mut conn = self.get_stream()?;
        let response = rt.block_on(requests::filter_response(&mut conn, request))?;
        self.conn.set(Some(conn));
        Ok(response)
    }

    fn play_game(&self, rt: &Runtime, count: u32) -> io::Result<()> {
        use crate::agent::NextRequest;
        use futures::sink::SinkExt;

        let (mut sink, mut stream) = self.get_stream()?.split();
        let mut agent = self.get_agent()?;
        let mut producer = self.get_producer()?;

        let start_hook = match agent.on_start_hook() {
            NextRequest::Agent(req) => req,
            NextRequest::Observation => producer.next().unwrap(),
        };

        let astream = producer::StreamAgent::new(&mut agent, &mut producer, &mut stream);
        let msink = std::sync::Arc::new(tokio::sync::Mutex::new(&mut sink));
        rt.block_on({
            let msink = std::sync::Arc::clone(&msink);
            async move {
                msink
                    .lock()
                    .await
                    .send(Request::with_id(start_hook, count + 1))
                    .await
            }
        })?;

        rt.block_on(astream.enumerate().for_each_concurrent(8, |(id, request)| {
            let msink = std::sync::Arc::clone(&msink);
            async move {
                log::trace!("sending response");
                match msink
                    .lock()
                    .await
                    .send(Request::with_id(request, id as u32))
                    .await
                {
                    Ok(()) => {}
                    Err(e) => log::error!("{:?}", e),
                };
            }
        }));
        agent.on_close_hook();

        self.agent.set(Some(agent));
        self.conn.set(Some(sink.reunite(stream).unwrap()));
        self.producer.set(Some(producer));

        Ok(())
    }
}

mod requests {
    use std::io;
    use std::net::SocketAddrV4;

    use futures::{
        future::{ready, Ready},
        sink::SinkExt,
        stream::{Stream, StreamExt},
    };
    use rsc2_pb::{
        codec::{from_framed, SC2ProtobufClient},
        sc2_api::{Request, Response},
    };
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
}
