use std::{io, time::Duration};

use futures::{SinkExt, StreamExt};
use log::info;
use rsc2::{
    connect, protocol,
    state_machine::{Core, InGame},
    Connection,
};
use tokio::time;

fn player_setup() -> Vec<protocol::PlayerSetup> {
    let mut p1 = protocol::PlayerSetup::default();
    p1.set_type(protocol::PlayerType::Participant);
    p1.set_race(protocol::Race::Terran);
    p1.player_name = Some("yolo, in the game".into());

    let mut p2 = protocol::PlayerSetup::default();
    p2.set_type(protocol::PlayerType::Computer);
    p2.set_race(protocol::Race::Terran);
    p2.set_difficulty(protocol::Difficulty::Easy);
    p2.player_name = Some("sentient cheese dip".into());

    vec![p1, p2]
}

fn local_map(path: impl Into<String>) -> protocol::request_create_game::Map {
    protocol::request_create_game::Map::LocalMap(protocol::LocalMap {
        map_path: Some(path.into()),
        map_data: None,
    })

    // protocol::request_create_game::Map::
}

async fn join_game(sm: &mut Core) -> io::Result<(InGame<'_>, Connection)> {
    use protocol::request_join_game::Participation;

    let mut connection = connect("127.0.0.1:8000").await?;

    let mut create_game = protocol::RequestCreateGame::default();
    create_game.player_setup = player_setup();
    create_game.map = Some(local_map(
        r"C:\Program Files (x86)\StarCraft II\Maps\EphemeronLE.SC2Map",
    ));
    create_game.realtime = Some(true);

    let state = sm
        .launched()
        .unwrap()
        .create_game(&mut connection, create_game)
        .await?
        .expect("couldn't create game");

    let mut join_game = protocol::RequestJoinGame::default();
    join_game.participation = Some(Participation::Race(protocol::Race::Terran as i32));
    join_game.options = Some(protocol::InterfaceOptions {
        raw: Some(true),
        ..Default::default()
    });
    let state = state.join_game(&mut connection, join_game).await?.unwrap();
    Ok((state, connection))
}

#[derive(Default)]
struct GameState {
    common: protocol::PlayerCommon,
    allies: Vec<protocol::Unit>,
    enemies: Vec<protocol::Unit>,
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
                self.enemies = units
                    .iter()
                    .filter(|unit| unit.alliance() == protocol::Alliance::Enemy)
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
    std::env::set_var("RUST_LOG", "info");

    pretty_env_logger::init_timed();

    let mut sm = Core::default();
    let (state, mut connection) = join_game(&mut sm).await?;

    let mut gameloop = state.stream(&mut connection);
    let mut idx = 0;

    let mut gs = GameState::default();

    loop {
        if idx >= 3 {
            break Ok(());
        }
        dbg!(idx);
        if idx == 0 {
            info!("Requesting observation");
            let mut req = protocol::Request::default();
            req.request = Some(protocol::request::Request::GameInfo(
                protocol::RequestGameInfo {},
            ));
            gameloop.send(req).await?;
            let response = match gameloop.next().await {
                Some(futures::future::Either::Right(_)) => break Ok(()), // game ended
                None => break Err(io::Error::new(io::ErrorKind::Other, "stream ended")),
                Some(futures::future::Either::Left(response)) => response,
            }?;

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

            time::sleep(Duration::from_secs(4)).await;
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
            Some(futures::future::Either::Right(_)) => break Ok(()), // game ended
            None => break Err(io::Error::new(io::ErrorKind::Other, "stream ended")),
            Some(futures::future::Either::Left(response)) => response,
        }?;

        gs.update(&response);

        let actions = gs.on_step();

        if !actions.is_empty() {
            let mut req = protocol::Request::default();
            req.request = Some(protocol::request::Request::Action(
                protocol::RequestAction { actions },
            ));

            info!("Sending action");
            dbg!(&req);

            gameloop.send(req).await?;
        }

        idx += 1;
    }
}
