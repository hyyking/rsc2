use std::io;

use futures::{SinkExt, StreamExt};
use rsc2::{connect, protocol, state_machine};

fn player_setup() -> Vec<protocol::PlayerSetup> {
    let mut p1 = protocol::PlayerSetup::default();
    p1.set_type(protocol::PlayerType::Participant);
    p1.set_race(protocol::Race::Terran);
    p1.player_name = Some("yolo in the game".into());

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();

    let mut connection = connect("127.0.0.1:8000").await?;
    let state = state_machine::init();

    let mut create_game = protocol::RequestCreateGame::default();
    create_game.player_setup = player_setup();
    create_game.map = Some(local_map("KingsCoveLE.SC2Map"));
    create_game.realtime = Some(true);

    let state = state
        .create_game(&mut connection, create_game)
        .await?
        .expect("couldn't create game");

    use protocol::request_join_game::Participation;
    let mut join_game = protocol::RequestJoinGame::default();
    join_game.participation = Some(Participation::Race(protocol::Race::Terran as i32));
    join_game.options = Some(protocol::InterfaceOptions {
        raw: Some(true),
        ..Default::default()
    });

    let state = state
        .join_game(&mut connection, join_game)
        .await?
        .expect("couldn't join game");

    let mut gameloop = state.stream(&mut connection);
    let mut idx = 0;
    loop {
        let request = protocol::RequestObservation {
            disable_fog: Some(false),
            game_loop: Some(idx),
        };
        let mut req = protocol::Request::default();
        req.id = Some(0);
        req.request = Some(protocol::request::Request::Observation(request));

        gameloop.send(req).await?;

        let response = match gameloop.next().await {
            Some(futures::future::Either::Right(_)) => break (Ok(())), // game ended
            None => break Err(io::Error::new(io::ErrorKind::Other, "stream ended")),
            Some(futures::future::Either::Left(response)) => response,
        };

        let protocol::Response { response, .. } = response?;
        use protocol::response;
        match response.unwrap() {
            response::Response::Observation(obs) => {
                let protocol::ObservationRaw { player, units, .. } =
                    obs.observation.and_then(|obs| obs.raw_data).unwrap();
                dbg!(player);
                dbg!(units
                    .iter()
                    .filter(|unit| unit.alliance() == protocol::Alliance::Self_)
                    .collect::<Vec<_>>());
            }
            _ => {}
        }

        idx += 1;
    }
}
