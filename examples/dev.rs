use rsc2::api::raw::{NewRawAgent, RawAgent, RawRequestGame};
use rsc2::hook::NextRequest;
use rsc2::pb::{api, prelude::*};

use std::pin::Pin;

struct Bot;

impl RawAgent for Bot {
    fn on_start(self: Pin<&mut Self>, _: api::Response) -> NextRequest {
        NextRequest::Observation
    }
    fn on_response(self: Pin<&mut Self>, _: api::Response) -> NextRequest {
        let actions = vec![api::Action {
            action_raw: None,
            action_feature_layer: None,
            action_render: None,
            action_ui: None,
            action_chat: Some(api::ActionChat {
                channel: Some(1),
                message: Some("Hello World".into()),
            }),
            game_loop: Some(0),
        }];
        NextRequest::Agent(api::request::Request::Action(api::RequestAction {
            actions,
        }))
    }
    fn on_end(&mut self) {}
}

#[rsc2::run]
fn game() -> std::io::Result<u32> {
    RawRequestGame::new(
        NewRawAgent(Bot {}),
        api::RequestCreateGame::default_config(),
        api::RequestJoinGame::default_config(),
    )
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();
    let request_count = game()?;
    println!("requests {}", request_count);
    Ok(())
}
