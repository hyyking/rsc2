use std::io;

use crate::{ingame::InGameLoop, Connection};

use futures::{sink::SinkExt, stream::StreamExt};
use rsc2_pb::protocol::{self, response::Response, Status};

macro_rules! server_call {
    ($conn:ident, $req:ident, $resp:path) => {
        async {
            let _: &mut Connection = $conn;
            $conn.send($req).await?;
            let res = match $conn.next().await {
                Some(Ok(res)) => res,
                Some(Err(err)) => return Err(err),
                None => return Ok(None),
            };

            let status = res.status();
            let id = res.id();
            let $crate::protocol::Response {
                response, error, ..
            } = &res;

            error
                .iter()
                .for_each(|err| warn!("response id: {} | err: {}", id, err));
            if matches!(status, Status::Quit | Status::Unknown) {
                info!("status: {:?}, interupting state machine", status);
                return Result::<_, io::Error>::Ok(None);
            }
            Ok(match response {
                Some($resp(response)) => {
                    if response.error.is_some() {
                        error!("response id: {} | err: {:?}", id, response.error());
                        None
                    } else {
                        Some(res)
                    }
                }
                _ => None,
            })
        }
    };
}

macro_rules! impl_from {
    ($from:ident -> $to:ident) => {
        impl<'a> From<$from<'a>> for $to<'a> {
            #[inline]
            fn from(rhs: $from<'a>) -> Self {
                $to(rhs.0)
            }
        }
    };
}

pub enum Core {
    Launched {},
    InitGame {},
    InGame {},
    InReplay {},
    Ended {},
}

impl Core {
    pub const fn init() -> Self {
        Self::Launched {}
    }
    pub fn launched(&mut self) -> Option<Launched<'_>> {
        match self {
            Self::Launched { .. } => Some(Launched::from(self)),
            _ => None,
        }
    }
    pub fn replace(&mut self, new: Self) -> Self {
        let (a, b) = (&self, &new);
        debug_assert!(
            // Launched
            matches!((a, b), (Self::Launched { .. }, Self::InitGame { .. }))
                || matches!((a, b), (Self::Launched { .. }, Self::InGame { .. }))
                || matches!((a, b), (Self::Launched { .. }, Self::InReplay { .. }))
                // InitGame
                || matches!((a, b), (Self::InitGame { .. }, Self::InGame { .. }))
                // InGame
                || matches!((a, b), (Self::InGame { .. }, Self::InGame { .. }))
                || matches!((a, b), (Self::InGame { .. }, Self::Ended { .. }))
                // InReplay
                || matches!((a, b), (Self::InReplay { .. }, Self::InReplay { .. }))
                || matches!((a, b), (Self::InReplay { .. }, Self::Ended { .. }))
                // Ended
                || matches!((a, b), (Self::Ended { .. }, Self::Launched { .. }))
                || matches!((a, b), (Self::Ended { .. }, Self::InGame { .. }))
        );
        std::mem::replace(self, new)
    }
}
impl Default for Core {
    fn default() -> Self {
        Self::init()
    }
}

#[repr(transparent)]
pub struct Launched<'a>(&'a mut Core);
impl<'a> From<&'a mut Core> for Launched<'a> {
    fn from(rhs: &'a mut Core) -> Self {
        Launched(rhs)
    }
}

#[repr(transparent)]
pub struct InitGame<'a>(&'a mut Core);
impl_from!(Launched -> InitGame);

#[repr(transparent)]
pub struct InGame<'a>(&'a mut Core);
impl_from!(Launched -> InGame);
impl_from!(InitGame -> InGame);

#[repr(transparent)]
pub struct InReplay<'a>(&'a mut Core);
impl_from!(Launched -> InReplay);

#[repr(transparent)]
pub struct Ended<'a>(&'a mut Core);
impl_from!(InGame -> Ended);
impl_from!(InReplay -> Ended);

impl<'a> Launched<'a> {
    pub fn core(&mut self) -> &mut Core {
        self.0
    }
    pub async fn create_game(
        self,
        framed: &mut Connection,
        data: protocol::RequestCreateGame,
    ) -> io::Result<Option<InitGame<'a>>> {
        let resp = server_call!(framed, data, Response::CreateGame).await?;
        Ok(resp.map(|_| {
            self.0.replace(Core::InitGame {});
            InitGame::from(self)
        }))
    }
    pub async fn join_game(
        self,
        framed: &mut Connection,
        data: protocol::RequestJoinGame,
    ) -> io::Result<Option<InGame<'a>>> {
        let resp = server_call!(framed, data, Response::JoinGame).await?;
        Ok(resp.map(|_| {
            self.0.replace(Core::InGame {});
            InGame::from(self)
        }))
    }
    pub async fn join_replay(
        self,
        framed: &mut Connection,
        data: protocol::RequestStartReplay,
    ) -> io::Result<Option<InReplay<'a>>> {
        let resp = server_call!(framed, data, Response::StartReplay).await?;
        Ok(resp.map(|_| {
            self.0.replace(Core::InReplay {});
            InReplay::from(self)
        }))
    }
}
impl<'a> InitGame<'a> {
    pub fn core(&mut self) -> &mut Core {
        self.0
    }
    pub async fn join_game(
        self,
        framed: &mut Connection,
        data: protocol::RequestJoinGame,
    ) -> io::Result<Option<InGame<'a>>> {
        let resp = server_call!(framed, data, Response::JoinGame).await?;
        Ok(resp.map(|_| {
            self.0.replace(Core::InGame {});
            InGame::from(self)
        }))
    }
}

impl<'a> InGame<'a> {
    pub fn core(&mut self) -> &mut Core {
        self.0
    }
    pub fn stream(self, stream: &mut Connection) -> InGameLoop<'a, '_> {
        let framed = unsafe { std::pin::Pin::new_unchecked(stream) };
        InGameLoop::new(self, framed)
    }
}
