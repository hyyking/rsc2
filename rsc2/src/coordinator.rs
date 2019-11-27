use std::{
    cell::Cell,
    io,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::hook::{AgentHook, NextRequest};
use crate::Commands;

use futures::{lock::Mutex, FutureExt, SinkExt, StreamExt};
use rsc2_pb::{api as pb, codec::SC2ProtobufClient, validate_status};
use tokio::{runtime::Runtime, timer::Interval};

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
macro_rules! arc_mutex {
    ($var: ident) => {{
        Arc::new(Mutex::new($var))
    }};
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

struct CoordinatorConfig {
    interval: Duration,
    core_threads: usize,
}
impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(50),
            core_threads: 4,
        }
    }
}

pub struct Coordinator<A>
where
    A: AgentHook + 'static,
{
    sm: StateMachine,
    config: CoordinatorConfig,

    conn: Cell<Option<SC2ProtobufClient>>,
    agent: Cell<Option<A>>,
}

impl<A> Coordinator<A>
where
    A: AgentHook + 'static,
{
    pub fn new() -> Self {
        Self {
            sm: StateMachine::new(),
            config: CoordinatorConfig::default(),

            conn: Cell::default(),
            agent: Cell::default(),
        }
    }

    pub fn run<Iter, Item>(&self, elements: Iter) -> io::Result<u32>
    where
        Item: Into<Commands<A>>,
        Iter: IntoIterator<Item = Item>,
    {
        let mut request_count: u32 = 0;
        let rt = tokio::runtime::Builder::new()
            .core_threads(self.config.core_threads)
            .name_prefix("rsc2-coordinator-thread")
            .build()?;

        for element in elements.into_iter().map(|el| el.into()) {
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
                        self.send_request(&rt, pb::Request::with_id(request, request_count))?;
                    validate_status!(response.status => pb::Status::InitGame)?;

                    self.sm.initgame()
                }
                Commands::JoinGame { request, agent } => {
                    log::debug!("Commands::JoinGame");
                    self.agent.set(Some(agent));

                    let response =
                        self.send_request(&rt, pb::Request::with_id(request, request_count))?;
                    validate_status!(response.status => pb::Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::StartReplay { request, agent } => {
                    log::debug!("Commands::StartReplay");
                    self.agent.set(Some(agent));

                    let response =
                        self.send_request(&rt, pb::Request::with_id(request, request_count))?;
                    validate_status!(response.status => pb::Status::InReplay)?;

                    self.sm.inreplay();
                    request_count = self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::RestartGame => {
                    log::debug!("Commands::RestartGame");
                    let response = self.send_request(
                        &rt,
                        pb::Request::with_id(pb::RequestRestartGame {}, request_count),
                    )?;
                    validate_status!(response.status => pb::Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(&rt, request_count)?;
                    self.sm.ended();
                }
                Commands::LeaveGame => {
                    log::debug!("Commands::LeaveGame");
                    let response = self.send_request(
                        &rt,
                        pb::Request::with_id(pb::RequestLeaveGame {}, request_count),
                    )?;
                    validate_status!(response.status => pb::Status::Launched)?;

                    self.sm.reset();
                    self.sm.launched();
                }
                Commands::QuitGame => {
                    log::debug!("Commands::QuitGame");
                    let response = self.send_request(
                        &rt,
                        pb::Request::with_id(pb::RequestQuit {}, request_count),
                    )?;
                    validate_status!(response.status => pb::Status::Quit)?;

                    self.sm.reset();
                }
            }
        }
        rt.shutdown_now();
        Ok(request_count)
    }
    /// {{{
    fn validate(&self, command: &Commands<A>) -> io::Result<()> {
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
            Commands::RestartGame => {
                is_launched()?;
                if !self.sm.is_ended() || !self.sm.is_ingame() {
                    return fast_err("Cannot restart a game while one is running");
                }
            }
            Commands::LeaveGame => {
                is_launched()?;
                if !self.sm.is_ended() {
                    return fast_err("Cannot leave a game while one is running");
                }
            }
            Commands::QuitGame => {
                is_launched()?;
            }
        }
        Ok(())
    }
    /// }}}

    fn send_request(&self, rt: &Runtime, request: pb::Request) -> io::Result<pb::Response> {
        log::trace!("send_request");
        let mut conn = self.get_ressource(false, true).1.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Engine is not connected to a SC2 instance",
            )
        })?;
        let response = rt.block_on(requests::filter_response(&mut conn, request))?;
        self.reset_ressource(None, Some(conn));
        Ok(response)
    }

    fn play_game(&self, rt: &Runtime, count: u32) -> io::Result<u32> {
        let (agent, ss) = self.get_ressource(true, true);

        let (sink, stream) = ss
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotConnected,
                    "Engine is not connected to a SC2 instance",
                )
            })?
            .split();
        let agent = agent
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Engine has no agent to run"))?;

        let producer = A::build_producer();
        let mut timer = Interval::new_interval(self.config.interval);

        let msink = arc_mutex!(sink);
        let mstream = arc_mutex!(stream);
        let magent = arc_mutex!(agent);
        let mprod = arc_mutex!(producer);
        let count = Arc::new(AtomicU32::new(count));

        let __: Result<(), io::Error> = rt.block_on({
            let stream = Arc::clone(&mstream);
            let agent = Arc::clone(&magent);
            let sink = Arc::clone(&msink);
            let prod = Arc::clone(&mprod);
            let count = Arc::clone(&count);

            async move {
                let start_hook = match agent.lock().await.on_start_hook() {
                    NextRequest::Agent(req) => req,
                    NextRequest::Observation => prod.lock().await.next().unwrap(),
                };
                sink.lock()
                    .await
                    .send(pb::Request::with_id(
                        start_hook,
                        count.fetch_add(1, Ordering::Acquire) + 1,
                    ))
                    .await?;

                let ended = Arc::new(AtomicBool::default());
                while let Some(_) = timer.next().await {
                    if ended.load(Ordering::Acquire) {
                        break;
                    }

                    let (stream_remote, stream_handle) = {
                        let stream = Arc::clone(&stream);
                        let count = Arc::clone(&count);
                        async move {
                            let mut stream = stream.lock().await;
                            log::trace!("receiver loop {}", count.load(Ordering::SeqCst));
                            let mut conn = stream.by_ref().filter(|resp| {
                                requests::id_filter(resp, Some(count.load(Ordering::Acquire)))
                            });
                            conn.next().await
                        }
                    }
                    .remote_handle();

                    let (agent_remote, agent_handle) = {
                        let agent = Arc::clone(&agent);
                        let ended = Arc::clone(&ended);
                        async move {
                            match stream_handle.await {
                                Some(Ok(obs)) => {
                                    if validate_status!(obs.status => pb::Status::Ended).is_ok() {
                                        ended.store(true, Ordering::Release);
                                        return None;
                                    }
                                    Some(agent.lock().await.on_step_hook(obs))
                                }
                                Some(Err(err)) => {
                                    log::error!("stream err: {}", err);
                                    None
                                }
                                None => {
                                    ended.store(true, Ordering::Release);
                                    return None;
                                }
                            }
                        }
                    }
                    .remote_handle();

                    rt.spawn(stream_remote);
                    rt.spawn(agent_remote);
                    rt.spawn({
                        let sink = Arc::clone(&sink);
                        let prod = Arc::clone(&prod);
                        let count = Arc::clone(&count);
                        async move {
                            let request = match agent_handle.await {
                                Some(request) => request,
                                None => return,
                            };
                            let request = pb::Request::with_id(
                                match request {
                                    NextRequest::Agent(req) => req,
                                    NextRequest::Observation => prod
                                        .lock()
                                        .await
                                        .next()
                                        .expect("observation generator ended"),
                                },
                                count.fetch_add(1, Ordering::AcqRel) + 1,
                            );
                            log::trace!("sender loop {}", count.load(Ordering::SeqCst));
                            sink.lock().await.send(request).await.expect("response");
                        }
                    });
                }
                Ok(())
            }
        });
        __?; // propagate the error of the runtime

        let sink = unwrap_arc(msink)?.into_inner();
        let stream = unwrap_arc(mstream)?.into_inner();
        let mut agent = unwrap_arc(magent)?.into_inner();

        agent.on_close_hook();

        self.reset_ressource(Some(agent), sink.reunite(stream).ok());

        Ok(unwrap_arc(count)?.into_inner())
    }

    #[inline]
    fn reset_ressource(&self, agent: Option<A>, conn: Option<SC2ProtobufClient>) {
        match agent {
            a @ Some(_) => self.agent.set(a),
            _ => {}
        }
        match conn {
            c @ Some(_) => self.conn.set(c),
            _ => {}
        }
    }

    #[inline]
    fn get_ressource(&self, agent: bool, conn: bool) -> (Option<A>, Option<SC2ProtobufClient>) {
        match (agent, conn) {
            (true, true) => (self.agent.take(), self.conn.take()),
            (true, false) => (self.agent.take(), None),
            (false, true) => (None, self.conn.take()),
            (false, false) => (None, None),
        }
    }
}

#[inline(always)]
fn unwrap_arc<T>(am: std::sync::Arc<T>) -> io::Result<T> {
    while Arc::strong_count(&am) != 1 {}
    Arc::try_unwrap(am)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "couldn't unwrap arc ressource"))
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
        api as pb,
        codec::{from_framed, SC2ProtobufClient},
    };
    use websocket_lite::ClientBuilder;

    pub(super) async fn init_connection(addr: SocketAddrV4) -> io::Result<SC2ProtobufClient> {
        let builder = ClientBuilder::new(&format!("ws://{}:{}/sc2api", addr.ip(), addr.port()))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid adress ws://{}:{}/sc2api", addr.ip(), addr.ip()),
                )
            })?;
        Ok(from_framed(
            builder
                .async_connect_insecure()
                .await
                .map_err(|e| *e.downcast::<io::Error>().unwrap())?,
        ))
    }

    pub(super) async fn filter_response(
        conn: &mut SC2ProtobufClient,
        request: pb::Request,
    ) -> io::Result<pb::Response> {
        let id = request.id;
        let mut conn = conn.filter(|resp| id_filter(resp, id));
        conn.send(request).await?;
        conn.next().await.ok_or_else(|| {
            io::Error::new(io::ErrorKind::ConnectionAborted, "No response for request")
        })?
    }

    #[inline(always)]
    pub(super) fn id_filter(
        resp: &<SC2ProtobufClient as Stream>::Item,
        id: Option<u32>,
    ) -> Ready<bool> {
        ready(resp.as_ref().ok().filter(|r| r.id == id).is_some())
    }
}
