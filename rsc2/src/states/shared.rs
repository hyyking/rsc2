use crate::agent;
use tokio::prelude::*;
use tokio::{codec::Framed, net::TcpStream};
use websocket::OwnedMessage;

type FramedStream = Framed<TcpStream, ::websocket::r#async::MessageCodec<OwnedMessage>>;

pub struct SharedState {
    pub conn: FramedStream,
    pub last_response: Option<OwnedMessage>,
    pub bot: Option<Box<dyn agent::Agent>>,
}

#[inline]
pub fn send_and_receive_sync(
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

impl SharedState {
    /// Synchronous messages because next state depends on this
    pub fn create_game(self, message: OwnedMessage) -> Self {
        info!("Creating game...");
        let (response, stream) = send_and_receive_sync(self.conn, message);
        debug!("{:?}", &response);
        Self {
            conn: stream,
            last_response: response,
            ..self
        }
    }
    pub fn join_game(self, message: OwnedMessage) -> Self {
        info!("Joining game...");
        let (response, stream) = send_and_receive_sync(self.conn, message);
        debug!("{:?}", &response);
        Self {
            conn: stream,
            last_response: response,
            ..self
        }
    }
    pub fn request_gamestate(self, message: OwnedMessage) -> Self {
        trace!("Requesting gamestate");
        let (response, stream) = send_and_receive_sync(self.conn, message);
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
