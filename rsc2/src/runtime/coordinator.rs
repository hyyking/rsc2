use std::cell::Cell;
use std::io;
use std::net::SocketAddrV4;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};

use crate::hook::{AgentHook, NextRequest};
use crate::runtime::{builder::CoordinatorConfig, state::StateMachine, Commands};

use futures::{lock::Mutex, FutureExt, SinkExt, StreamExt};
use rsc2_pb::{
    api as pb,
    codec::{from_framed, SC2ProtobufClient},
    validate_status,
};
use tokio::{runtime::TaskExecutor, timer::Interval};
use websocket_lite::ClientBuilder;

pub struct Coordinator<A> {
    sm: StateMachine,
    config: CoordinatorConfig,

    conn: Cell<Option<SC2ProtobufClient>>,
    agent: Cell<Option<A>>,
}

impl<A: AgentHook + 'static> Default for Coordinator<A> {
    fn default() -> Self {
        Self {
            sm: StateMachine::default(),
            config: CoordinatorConfig::default(),
            conn: Cell::default(),
            agent: Cell::default(),
        }
    }
}

impl<A: AgentHook + 'static> Coordinator<A> {
    /// Build a new coordinator with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a handle to the coordinator's runtime. Tasks spawned will be concurrently run alongside
    /// the coordinators requests and responses. It is advised to only spawn tasks within your agent.
    // TODO: Create a handle that can be used to spawn tasks during the game.
    pub fn executor(&self) -> TaskExecutor {
        self.config.runtime.executor()
    }

    pub fn run<Iter, Item>(&self, elements: Iter) -> io::Result<u32>
    where
        Item: Into<Commands<A>>,
        Iter: IntoIterator<Item = Item>,
    {
        let mut request_count: u32 = 0;

        for element in elements.into_iter().map(|el| el.into()) {
            request_count += 1;
            self.sm.validate(&element)?;
            match element {
                Commands::Launched { socket, .. } => {
                    log::debug!("Commands::Launched");
                    self.connect(socket)?;
                    self.sm.launched()
                }
                Commands::CreateGame { request } => {
                    log::debug!("Commands::CreateGame");

                    let response = self.send(pb::Request::with_id(request, request_count))?;
                    validate_status!(response.status => pb::Status::InitGame)?;

                    self.sm.initgame()
                }
                Commands::JoinGame { request, agent } => {
                    log::debug!("Commands::JoinGame");
                    self.agent.set(Some(agent));

                    let response = self.send(pb::Request::with_id(request, request_count))?;
                    validate_status!(response.status => pb::Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(request_count)?;
                    self.sm.ended();
                }
                Commands::StartReplay { request, agent } => {
                    log::debug!("Commands::StartReplay");
                    self.agent.set(Some(agent));

                    let response = self.send(pb::Request::with_id(request, request_count))?;
                    validate_status!(response.status => pb::Status::InReplay)?;

                    self.sm.inreplay();
                    request_count = self.play_game(request_count)?;
                    self.sm.ended();
                }
                Commands::RestartGame => {
                    log::debug!("Commands::RestartGame");
                    let response = self.send(pb::Request::with_id(
                        pb::RequestRestartGame {},
                        request_count,
                    ))?;
                    validate_status!(response.status => pb::Status::InGame)?;

                    self.sm.ingame();
                    request_count = self.play_game(request_count)?;
                    self.sm.ended();
                }
                Commands::LeaveGame => {
                    log::debug!("Commands::LeaveGame");
                    let response =
                        self.send(pb::Request::with_id(pb::RequestLeaveGame {}, request_count))?;
                    validate_status!(response.status => pb::Status::Launched)?;

                    self.sm.reset();
                    self.sm.launched();
                }
                Commands::QuitGame => {
                    log::debug!("Commands::QuitGame");
                    let response =
                        self.send(pb::Request::with_id(pb::RequestQuit {}, request_count))?;
                    validate_status!(response.status => pb::Status::Quit)?;

                    self.sm.reset();
                }
            }
        }
        Ok(request_count)
    }

    fn play_game(&self, count: u32) -> io::Result<u32> {
        macro_rules! arc_mutex {
            ($var: ident) => {{
                Arc::new(Mutex::new($var))
            }};
        }
        let rt = &self.config.runtime;
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
                            log::trace!("receiver loop: {}", count.load(Ordering::Acquire));
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
                                    NextRequest::Observation => prod.lock().await.next().unwrap(),
                                },
                                count.fetch_add(1, Ordering::AcqRel) + 1,
                            );
                            log::trace!("sender loop: {}", count.load(Ordering::Acquire));
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
}

impl<A: AgentHook + 'static> Coordinator<A> {
    fn connect(&self, addr: SocketAddrV4) -> io::Result<()> {
        let builder = ClientBuilder::new(&format!("ws://{}:{}/sc2api", addr.ip(), addr.port()))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid adress ws://{}:{}/sc2api", addr.ip(), addr.ip()),
                )
            })?;
        let framed = self
            .config
            .runtime
            .block_on(builder.async_connect_insecure())
            .map_err(|e| *e.downcast::<io::Error>().unwrap())?;
        self.conn.set(Some(from_framed(framed)));
        Ok(())
    }

    fn send(&self, request: pb::Request) -> io::Result<pb::Response> {
        let mut conn = self.get_ressource(false, true).1.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Engine is not connected to a SC2 instance",
            )
        })?;
        let response = self
            .config
            .runtime
            .block_on(requests::filter_response(&mut conn, request))?;
        self.reset_ressource(None, Some(conn));
        Ok(response)
    }
}

// TODO: Move this to a ressource handler
impl<A: AgentHook + 'static> Coordinator<A> {
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
        let mut ressource = (None, None);
        if agent {
            ressource.0 = self.agent.take();
        }
        if conn {
            ressource.1 = self.conn.take();
        }
        ressource
    }
}

impl<A: AgentHook + 'static> From<CoordinatorConfig> for Coordinator<A> {
    fn from(config: CoordinatorConfig) -> Self {
        Self {
            config,
            ..Default::default()
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

    use futures::{
        future::{ready, Ready},
        sink::SinkExt,
        stream::{Stream, StreamExt},
    };
    use rsc2_pb::{api as pb, codec::SC2ProtobufClient};

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
