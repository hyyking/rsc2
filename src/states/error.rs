use crate::sc2_api;
use prost::Message;

pub trait HandleEncodeError {
    fn unwrap_or_quit(self) -> websocket::OwnedMessage;
}

impl<S> HandleEncodeError for Result<websocket::OwnedMessage, S>
where
    S: std::error::Error + Sized,
{
    fn unwrap_or_quit(self) -> websocket::OwnedMessage {
        match self {
            Ok(message) => message,
            Err(err) => {
                let mut buff = vec![];
                let quit = sc2_api::Request {
                    id: None,
                    request: Some(sc2_api::request::Request::Quit(sc2_api::RequestQuit {})),
                };
                if let Err(e) = quit.encode(&mut buff) {
                    error!(
                        "Couldn't quit the game. You need to manually close the SC2 instance..."
                    );
                    panic!("Original Encoding: {}, QuitGame encoding: {}", err, e);
                };
                websocket::OwnedMessage::Binary(buff)
            }
        }
    }
}
