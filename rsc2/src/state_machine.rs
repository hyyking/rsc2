use std::io;

use crate::{ingame::InGameLoop, Connection};

use futures::{sink::SinkExt, stream::StreamExt};
use rsc2_pb::protocol::{self, response::Response, Status};

macro_rules! state_machine {
    (struct $state_machine:ident<S: $marker:ident> { $($state:ident => {$((fn $method:ident -> $next_state:ident)),*}),+}) => {
        #[derive(Clone, Copy, Eq, PartialEq)]
        pub struct $state_machine<S: $marker> {
            _state: ::core::marker::PhantomData<S>,
        }
        $(
            #[derive(Clone, Copy, Eq, PartialEq)]
            pub struct $state;
            impl $marker for $state {}
            impl $state_machine<$state> {
                $(
                    pub(crate) fn $method(self) -> $state_machine<$next_state> {
                        $state_machine { _state: ::core::marker::PhantomData }
                    }
                )*
            }
        )*
    }
}
macro_rules! server_call {
    ($framed:ident, $data:ident, $transition:expr, $variant:path) => {{
        let _: &mut Connection = $framed;
        $framed.send($data).await?;
        match $framed.next().await.transpose()? {
            Some(response) => {
                let status = response.status();
                let id = response.id();
                let $crate::protocol::Response {
                    response, error, ..
                } = response;
                error
                    .into_iter()
                    .for_each(|err| warn!("response id: {} | err: {}", id, err));
                if matches!(status, Status::Quit | Status::Unknown) {
                    info!("status: {:?}, interupting state machine", status);
                    return Ok(None);
                }
                if let Some($variant(response)) = response {
                    if response.error.is_some() {
                        error!("response id: {} | err: {:?}", id, response.error());
                        return Ok(None);
                    }
                }
                info!("transitioning to status: {:?}", status);
                Ok(Some($transition))
            }
            None => Ok(None),
        }
    }};
}

#[marker]
pub trait State {}

state_machine!(struct Engine<S: State> {
    Launched => {
        (fn as_init_game -> InitGame),
        (fn as_in_game -> InGame),
        (fn as_in_replay -> InReplay)
    },
    InitGame => {
        (fn as_in_game -> InGame)
    },
    InGame => {
        (fn as_in_game -> InGame),
        (fn as_ended -> Ended)
    },
    InReplay => {
        (fn as_in_replay -> InReplay),
        (fn as_ended -> Ended)
    },
    Ended => {
        (fn as_launched -> Launched),
        (fn as_ingame -> InGame)
    }
});

type EngineResult<T> = io::Result<Option<Engine<T>>>;

pub fn init() -> Engine<Launched> {
    Engine {
        _state: ::core::marker::PhantomData,
    }
}

impl Engine<Launched> {
    pub async fn create_game(
        self,
        framed: &mut Connection,
        data: protocol::RequestCreateGame,
    ) -> EngineResult<InitGame> {
        server_call!(framed, data, self.as_init_game(), Response::CreateGame)
    }
    pub async fn join_game(
        self,
        framed: &mut Connection,
        data: protocol::RequestJoinGame,
    ) -> EngineResult<InGame> {
        server_call!(framed, data, self.as_in_game(), Response::JoinGame)
    }
    pub async fn join_replay(
        self,
        framed: &mut Connection,
        data: protocol::RequestStartReplay,
    ) -> EngineResult<InReplay> {
        server_call!(framed, data, self.as_in_replay(), Response::StartReplay)
    }
}
impl Engine<InitGame> {
    pub async fn join_game(
        self,
        framed: &mut Connection,
        data: protocol::RequestJoinGame,
    ) -> EngineResult<InGame> {
        server_call!(framed, data, self.as_in_game(), Response::JoinGame)
    }
}

impl Engine<InGame> {
    pub fn stream(self, stream: &mut Connection) -> InGameLoop<'_> {
        let framed = unsafe { std::pin::Pin::new_unchecked(stream) };
        InGameLoop::new(self, framed)
    }
}
impl Engine<InReplay> {}
impl Engine<Ended> {}

impl std::process::Termination for Engine<Ended> {
    fn report(self) -> i32 {
        0
    }
}
