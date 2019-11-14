use rsc2::agent;
use rsc2::builder::RawRequestGame;
use rsc2::Coordinator;
use rsc2::{pb_prelude::*, sc2_api};

struct Bot;

impl agent::Agent for Bot {
    fn on_step(&mut self, _obs: &sc2_api::Observation) -> Option<sc2_api::RequestAction> {
        let actions = vec![sc2_api::Action {
            action_raw: None,
            action_feature_layer: None,
            action_render: None,
            action_ui: None,
            action_chat: Some(sc2_api::ActionChat {
                channel: Some(1),
                message: Some("Hello World".into()),
            }),
            game_loop: None,
        }];
        Some(sc2_api::RequestAction { actions })
    }
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();

    let mut c = Coordinator::new();
    let requests = c.run(RawRequestGame::new(
        Bot {},
        sc2_api::RequestCreateGame::default_config(),
        sc2_api::RequestJoinGame::default_config(),
    ))?;
    println!("requests {:?}", requests);
    Ok(())
}
