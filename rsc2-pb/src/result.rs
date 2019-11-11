use crate::sc2_api;
use prost::EncodeError;
use websocket_lite::Message;

/// Result<OwnedMessage, EncodeError> type for prost encoding
#[derive(Debug)]
pub enum EncodeResult {
    EncodeOk(Message),
    EncodeErr(EncodeError),
}

impl EncodeResult {
    pub fn unwrap(self) -> Message {
        match self {
            EncodeResult::EncodeOk(message) => message,
            EncodeResult::EncodeErr(e) => {
                panic!("Called EncodeResult::unwrap() on an encoding error {:?}", e);
            }
        }
    }

    pub fn unwrap_or_quit(self) -> Message {
        use sc2_api::{request::Request::Quit, RequestQuit};
        use EncodeResult::*;
        match self {
            EncodeOk(ok_message) => ok_message,
            EncodeErr(err) => {
                if let EncodeOk(quit_message) = (sc2_api::Request {
                    id: None,
                    request: Some(Quit(RequestQuit {})),
                }
                .into())
                {
                    quit_message
                } else {
                    panic!("Couldn't quit the game. You need to manually close the SC2 instance...\nEncoding Err: {}", err);
                }
            }
        }
    }
}

impl<M> From<M> for EncodeResult
where
    M: prost::Message,
{
    fn from(query: M) -> Self {
        let mut buff = vec![];
        match query.encode(&mut buff) {
            Ok(_) => EncodeResult::EncodeOk(Message::binary(buff)),
            Err(e) => EncodeResult::EncodeErr(e),
        }
    }
}
