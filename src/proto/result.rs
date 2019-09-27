use crate::proto::sc2_api;
use prost::{EncodeError, Message};
use websocket::OwnedMessage;

/// Result<OwnedMessage, EncodeError> type for prost encoding
#[derive(Debug)]
pub enum EncodeResult {
    EncodeSuccess(OwnedMessage),
    EncodeError(EncodeError),
}

impl EncodeResult {
    pub fn unwrap(self) -> OwnedMessage {
        match self {
            EncodeResult::EncodeSuccess(message) => message,
            EncodeResult::EncodeError(e) => {
                panic!("Called EncodeResult::unwrap() on an encoding error {:?}", e);
            }
        }
    }

    pub fn unwrap_or_quit(self) -> OwnedMessage {
        match self {
            EncodeResult::EncodeSuccess(ok_message) => ok_message,
            EncodeResult::EncodeError(err) => {
                match (sc2_api::Request {
                    id: None,
                    request: Some(sc2_api::request::Request::Quit(sc2_api::RequestQuit {})),
                }
                .into())
                {
                    EncodeResult::EncodeSuccess(quit_message) => quit_message,
                    EncodeResult::EncodeError(e) => {
                        error!(
                        "Couldn't quit the game. You need to manually close the SC2 instance..."
                    );
                        panic!(
                            "Original Encoding Err: {}\nQuitGame encoding Error: {}",
                            err, e
                        );
                    }
                }
            }
        }
    }
}

impl<M> From<M> for EncodeResult
where
    M: Message,
{
    fn from(query: M) -> Self {
        let mut buff = vec![];
        match query.encode(&mut buff) {
            Ok(_) => EncodeResult::EncodeSuccess(OwnedMessage::Binary(buff)),
            Err(e) => EncodeResult::EncodeError(e),
        }
    }
}
