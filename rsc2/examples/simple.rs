use std::io;

use futures::{SinkExt, StreamExt};
use rsc2::{
    connect, protocol,
    state_machine::{Core, InGame},
    Connection,
};

fn player_setup() -> Vec<protocol::PlayerSetup> {
    let mut p1 = protocol::PlayerSetup::default();
    p1.set_type(protocol::PlayerType::Participant);
    p1.set_race(protocol::Race::Terran);
    p1.player_name = Some("yolo, in the game".into());

    let mut p2 = protocol::PlayerSetup::default();
    p2.set_type(protocol::PlayerType::Computer);
    p2.set_race(protocol::Race::Terran);
    p2.set_difficulty(protocol::Difficulty::Easy);
    p2.player_name = Some("brainful cheese dip".into());

    vec![p1, p2]
}

fn local_map(path: impl Into<String>) -> protocol::request_create_game::Map {
    protocol::request_create_game::Map::LocalMap(protocol::LocalMap {
        map_path: Some(path.into()),
        map_data: None,
    })
}

async fn join_game(sm: &mut Core) -> io::Result<(InGame<'_>, Connection)> {
    use protocol::request_join_game::Participation;

    let mut connection = connect("127.0.0.1:8000").await?;

    let mut create_game = protocol::RequestCreateGame::default();
    create_game.player_setup = player_setup();
    create_game.map = Some(local_map("KingsCoveLE.SC2Map"));
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
                if unit.unit_type == Some(rsc2_pb::ids::UnitId::Scv as u32) {
                    return unit.tag;
                } else {
                    None
                }
            })
            .collect();

        let mut raw = protocol::ActionRaw::default();
        raw.action = Some(protocol::action_raw::Action::UnitCommand(
            protocol::ActionRawUnitCommand {
                ability_id: Some(rsc2_pb::ids::AbilityId::TauntTaunt as i32),
                unit_tags: scvs,
                queue_command: Some(false),
                target: None,
            },
        ));

        self.stepped = true;

        vec![protocol::Action {
            action_raw: Some(raw),
            ..Default::default()
        }]
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();
    let mut sm = Core::default();
    let (state, mut connection) = join_game(&mut sm).await?;

    let mut gameloop = state.stream(&mut connection);
    let mut idx = 0;

    let mut gs = GameState::default();
    loop {
        let mut req = protocol::Request::default();
        req.request = Some(protocol::request::Request::Observation(
            protocol::RequestObservation {
                disable_fog: Some(false),
                game_loop: Some(idx),
            },
        ));

        gameloop.send(req).await?;
        let response = match gameloop.next().await {
            Some(futures::future::Either::Right(_)) => break (Ok(())), // game ended
            None => break Err(io::Error::new(io::ErrorKind::Other, "stream ended")),
            Some(futures::future::Either::Left(response)) => response,
        }?;

        if let Some(protocol::response::Response::Action(ref action)) = response.response {
            dbg!(action.result().collect::<Vec<_>>());
        }
        gs.update(&response);

        let mut req = protocol::Request::default();
        req.request = Some(protocol::request::Request::Action(
            protocol::RequestAction {
                actions: gs.on_step(),
            },
        ));

        gameloop.send(req).await?;
        idx += 1;
    }
}
