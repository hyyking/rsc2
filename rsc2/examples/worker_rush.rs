use std::{io, time::Duration};

use futures::{SinkExt, StreamExt};
use log::info;
use rsc2::{
    prelude::{Difficulty, Player, Race, create_game},
    protocol,
    state_machine::Core,
};
use tokio::time;

#[derive(Default)]
struct GameState {
    common: protocol::PlayerCommon,
    allies: Vec<protocol::Unit>,
    start_location: Option<protocol::Point2D>,
    stepped: bool,
}

impl GameState {
    fn update(&mut self, response: &protocol::Response) {
        let protocol::Response { response, .. } = response;

        match response.as_ref().unwrap() {
            protocol::response::Response::Observation(obs) => {
                self.common = obs
                    .observation
                    .as_ref()
                    .and_then(|obs| obs.player_common.clone())
                    .unwrap();

                let protocol::ObservationRaw { units, .. } = obs
                    .observation
                    .as_ref()
                    .and_then(|obs| obs.raw_data.as_ref())
                    .unwrap();

                self.allies = units
                    .iter()
                    .filter(|unit| unit.alliance() == protocol::Alliance::Self_)
                    .cloned()
                    .collect();
            }
            _ => {}
        }
    }
    fn on_step(&mut self) -> Vec<protocol::Action> {
        if self.stepped {
            return vec![];
        }

        let scvs: Vec<_> = self
            .allies
            .iter()
            .filter_map(|unit| {
                if unit.unit_type == Some(45) {
                    return unit.tag;
                } else {
                    None
                }
            })
            .collect();

        if let Some(start_location) = self.start_location.clone() {
            let mut raw = protocol::ActionRaw::default();
            raw.action = Some(protocol::action_raw::Action::UnitCommand(
                protocol::ActionRawUnitCommand {
                    ability_id: Some(23),
                    unit_tags: scvs,
                    queue_command: Some(false),
                    target: Some(
                        protocol::action_raw_unit_command::Target::TargetWorldSpacePos(
                            start_location,
                        ),
                    ),
                },
            ));
            self.stepped = true;
            vec![protocol::Action {
                action_raw: Some(raw),
                ..Default::default()
            }]
        } else {
            vec![]
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();

    let mut sm = Core::default();

    let (state, mut connection) = create_game(
        &mut sm,
        "127.0.0.1:8000",
        [
            Player::participant("yolo, in the game", Race::Terran),
            Player::bot("sentient cheese dip", Race::Zerg, Difficulty::Easy),
        ],
        r"C:\Program Files (x86)\StarCraft II\Maps\EphemeronLE.SC2Map",
        true,
    )
    .await?;

    let mut gameloop = state.stream(&mut connection);
    let mut idx = 0;

    let mut gs = GameState::default();

    let _state = loop {
        log::trace!("Game loop iteration {idx}");

        if idx == 0 {
            info!("Requesting info");
            let mut req = protocol::Request::default();
            req.request = Some(protocol::request::Request::GameInfo(
                protocol::RequestGameInfo {},
            ));
            gameloop.send(req).await?;
            let response = match gameloop.next().await {
                Some(Ok(result)) => result,
                Some(Err(error)) => break Err(error),
                None => break Ok(gameloop.into_ended()),
            };

            if let Some(protocol::response::Response::GameInfo(protocol::ResponseGameInfo {
                start_raw:
                    Some(protocol::StartRaw {
                        ref start_locations,
                        ..
                    }),
                ..
            })) = response.response
            {
                gs.start_location = Some(start_locations[0].clone());
            }

            // start timer
            time::sleep(Duration::from_secs(3)).await;
        }

        info!("Requesting observation");
        let mut req = protocol::Request::default();
        req.request = Some(protocol::request::Request::Observation(
            protocol::RequestObservation {
                disable_fog: Some(false),
                game_loop: None, //Some(idx),
            },
        ));

        gameloop.send(req).await?;
        let response = match gameloop.next().await {
            Some(Ok(result)) => result,
            Some(Err(error)) => break Err(error),
            None => break Ok(gameloop.into_ended()),
        };

        gs.update(&response);

        let actions = gs.on_step();

        if !actions.is_empty() {
            let mut req = protocol::Request::default();
            req.request = Some(protocol::request::Request::Action(
                protocol::RequestAction { actions },
            ));

            info!("Sending action");
            gameloop.send(req).await?;
        }

        idx += 1;
    }?;
    log::info!("Game loop finished gracefully after {} iterations", idx);

    Ok(())
}
