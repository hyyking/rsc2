use rsc2::api::raw::{NewRawAgent, RawAgent, RawRequestGame};
use rsc2::hook::NextRequest;
use rsc2::pb::{api, prelude::*};
use rsc2::runtime::Coordinator;

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
            game_loop: None,
        }];
        NextRequest::Agent(api::request::Request::Action(api::RequestAction {
            actions,
        }))
    }
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();

    let c = Coordinator::new();
    let requests = c.run(RawRequestGame::new(
        NewRawAgent(Bot {}),
        api::RequestCreateGame::default_config(),
        api::RequestJoinGame::default_config(),
    ))?;
    println!("requests {:?}", requests);
    Ok(())
}
