use std::cell::Cell;
use std::io;
use std::net::SocketAddrV4;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};

use crate::hook::{AgentHook, NextRequest};
use crate::runtime::ressource::*;
use crate::runtime::{builder::CoordinatorConfig, state::StateMachine, Commands};

use futures::{lock::Mutex, SinkExt, StreamExt};
use rsc2_pb::{
    api as pb,
    codec::{from_framed, SC2ProtobufClient},
    validate_status,
};
use tokio::time::interval;
use websocket_lite::ClientBuilder;

/// Coordinator ...
pub struct Coordinator<A> {
    sm: StateMachine,
    config: CoordinatorConfig,

    conn: Cell<Option<SC2ProtobufClient>>,
    agent: Cell<Option<A>>,
}

impl<A: AgentHook + 'static> Coordinator<A> {
    /// Build a new coordinator with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Run an Iterator of [`Commands`](crate::runtime::Commands) to completion.
    ///
    /// # Panics
    ///
    /// The validity of the state switching is checked at runtime so it might produce errors
    /// alongside the stream's [`io::Error`](std::io::Error).
    pub fn run<Iter, Item>(&self, elements: Iter) -> io::Result<u32>
    where
        Item: Into<Commands<A>>,
        Iter: IntoIterator<Item = Item>,
    {
        let mut count: u32 = 0;

        for element in elements.into_iter().map(|el| el.into()) {
            count += 1;
            self.sm.validate(&element)?;
            match element {
                Commands::Launched { socket, .. } => {
                    log::debug!("Commands::Launched");

                    self.connect(socket)?;
                    self.sm.launched()
                }
                Commands::CreateGame { request } => {
                    log::debug!("Commands::CreateGame");

                    let response = self.send(pb::Request::with_id(request, count))?;
                    validate_status!(response.status => pb::Status::InitGame)?;

                    self.sm.initgame()
                }
                Commands::JoinGame { request, agent } => {
                    log::debug!("Commands::JoinGame");

                    self.agent.set(Some(agent));

                    let response = self.send(pb::Request::with_id(request, count))?;
                    validate_status!(response.status => pb::Status::InGame)?;

                    self.sm.ingame();
                    count = self.play_game(count)?;
                    self.sm.ended();
                }
                Commands::StartReplay { request, agent } => {
                    log::debug!("Commands::StartReplay");

                    self.agent.set(Some(agent));

                    let response = self.send(pb::Request::with_id(request, count))?;
                    validate_status!(response.status => pb::Status::InReplay)?;

                    self.sm.inreplay();
                    count = self.play_game(count)?;
                    self.sm.ended();
                }
                Commands::RestartGame => {
                    log::debug!("Commands::RestartGame");

                    let response =
                        self.send(pb::Request::with_id(pb::RequestRestartGame {}, count))?;
                    validate_status!(response.status => pb::Status::InGame)?;

                    self.sm.ingame();
                    count = self.play_game(count)?;
                    self.sm.ended();
                }
                Commands::LeaveGame => {
                    log::debug!("Commands::LeaveGame");

                    let response =
                        self.send(pb::Request::with_id(pb::RequestLeaveGame {}, count))?;
                    validate_status!(response.status => pb::Status::Launched)?;

                    self.sm.reset();
                    self.sm.launched();
                }
                Commands::QuitGame => {
                    log::debug!("Commands::QuitGame");

                    let response = self.send(pb::Request::with_id(pb::RequestQuit {}, count))?;
                    validate_status!(response.status => pb::Status::Quit)?;

                    self.sm.reset();
                }
            }
        }
        Ok(count)
    }

    // TODO: CLEAN UP
    fn play_game(&self, count: u32) -> io::Result<u32> {
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

        let lock = Arc::new(AtomicBool::new(false));
        let count = Arc::new(AtomicU32::new(count));

        let mut ig = InGameRessource {
            main: MainRessource {
                timer: rt.borrow().enter(|| interval(self.config.interval)),
                lock: lock.clone(),
            },
            loop_res: Arc::new(LoopRessource {
                reqr: RequestRessource {
                    count: count.clone(),
                    sink: Mutex::new(sink),
                    prod: Mutex::new(A::build_producer()),
                },
                resr: ResponseRessource {
                    count,
                    stream: Mutex::new(stream),
                },
                ager: AgentRessource {
                    lock,
                    agent: Mutex::new(agent),
                },
            }),
        };

        rt.borrow_mut().block_on(ingame::run(&mut ig))?;

        let LoopRessource { reqr, resr, ager } = unwrap_arc(ig.loop_res)?;
        let RequestRessource { sink, count, .. } = reqr;
        let stream = resr.stream.into_inner();
        let mut agent = ager.agent.into_inner();

        agent.on_close_hook();

        self.reset_ressource(Some(agent), sink.into_inner().reunite(stream).ok());

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
            .borrow_mut()
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
            .borrow_mut()
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

impl<A> std::fmt::Debug for Coordinator<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Coordinator")
            .field("state", &self.sm)
            .finish()
    }
}
impl<A> Default for Coordinator<A> {
    fn default() -> Self {
        Self {
            sm: StateMachine::default(),
            config: CoordinatorConfig::default(),
            conn: Cell::default(),
            agent: Cell::default(),
        }
    }
}
impl<A> From<CoordinatorConfig> for Coordinator<A> {
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

mod ingame {

    use super::*;
    use crate::runtime::ressource::{AgentRessource, InGameRessource, ResponseRessource};
    use std::ops::DerefMut;
    use std::pin::Pin;

    pub(super) async fn run<A: AgentHook + 'static>(ig: &mut InGameRessource<A>) -> io::Result<()> {
        let start_hook =
            match unsafe { Pin::new_unchecked(ig.loop_res.ager.agent.lock().await.deref_mut()) }
                .on_start_hook()
            {
                NextRequest::Agent(req) => req,
                NextRequest::Observation => ig.loop_res.reqr.prod.lock().await.next().unwrap(),
            };
        ig.loop_res
            .reqr
            .sink
            .lock()
            .await
            .send(pb::Request::with_id(
                start_hook,
                ig.loop_res.reqr.count.fetch_add(1, Ordering::Acquire) + 1,
            ))
            .await?;
        loop {
            let tick = ig.main.timer.tick().await;

            if ig.main.lock.load(Ordering::Acquire) {
                break;
            }

            tokio::spawn({
                let lr = ig.loop_res.clone();
                tokio::time::timeout_at(tick + std::time::Duration::from_millis(50), async move {
                    let request =
                        match ingame::agent_loop(ingame::response_loop(&lr.resr).await, &lr.ager)
                            .await
                        {
                            Some(request) => request,
                            None => return,
                        };
                    let request = pb::Request::with_id(
                        match request {
                            NextRequest::Agent(req) => req,
                            NextRequest::Observation => lr.reqr.prod.lock().await.next().unwrap(),
                        },
                        lr.reqr.count.fetch_add(1, Ordering::AcqRel) + 1,
                    );
                    log::trace!("sender loop: {}", lr.reqr.count.load(Ordering::Acquire));
                    lr.reqr
                        .sink
                        .lock()
                        .await
                        .send(request)
                        .await
                        .expect("response");
                })
            });
        }
        Ok(())
    }

    pub(super) async fn response_loop(
        resr: &ResponseRessource,
    ) -> Option<<SC2ProtobufClient as futures::Stream>::Item> {
        let mut stream = resr.stream.lock().await;
        log::trace!("receiver loop: {}", resr.count.load(Ordering::Acquire));
        let mut conn = stream
            .by_ref()
            .filter(|resp| requests::id_filter(resp, Some(resr.count.load(Ordering::Acquire))));
        conn.next().await
    }

    type RemoteStreamItem = Option<<SC2ProtobufClient as futures::Stream>::Item>;

    pub(super) async fn agent_loop<A: AgentHook + 'static>(
        response: RemoteStreamItem,
        ager: &AgentRessource<A>,
    ) -> Option<NextRequest> {
        match response {
            Some(Ok(obs)) => {
                if validate_status!(obs.status => pb::Status::Ended).is_ok() {
                    ager.lock.store(true, Ordering::Release);
                    return None;
                }
                let resp = unsafe { Pin::new_unchecked(ager.agent.lock().await.deref_mut()) }
                    .on_step_hook(obs);
                Some(resp)
            }
            Some(Err(err)) => {
                log::error!("stream err: {}", err);
                None
            }
            None => {
                ager.lock.store(true, Ordering::Release);
                return None;
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
