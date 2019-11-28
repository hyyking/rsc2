use rsc2::api::raw::{NewRawAgent, RawAgent, RawRequestGame};
use rsc2::hook::NextRequest;
use rsc2::pb::{api, prelude::*};

struct Bot;

impl RawAgent for Bot {
    fn on_response(&mut self, _: api::Response) -> NextRequest {
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
