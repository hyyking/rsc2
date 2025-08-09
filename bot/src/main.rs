mod queries;
mod store;
mod throughput;

use std::{io, sync::Arc, time::Duration};

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use log::info;
use rsc2::{
    prelude::{Difficulty, Player, Race, create_game},
    protocol,
    state_machine::Core,
};
use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
};

use crate::store::World;

struct Bot {
    world: World,
}

impl Bot {
    async fn new(db: Arc<Surreal<Client>>) -> Self {
        Self {
            world: World::new(db),
        }
    }

    async fn update(&mut self, response: protocol::Response) {
        let protocol::Response {
            response: Some(protocol::response::Response::Observation(obs)),
            ..
        } = response
        else {
            log::trace!("Received non-observation response");
            return;
        };

        if let Some(observation) = obs.observation.map(|obs| obs.raw_data).flatten() {
            self.world
                .register_observation_raw(observation)
                .await
                .unwrap();
        }
    }
}

async fn request_observation(gameloop: &mut rsc2::InGameListener<'_, '_>) -> Result<(), io::Error> {
    info!("Requesting observation");
    let mut req = protocol::Request::default();
    req.request = Some(protocol::request::Request::Observation(
        protocol::RequestObservation {
            disable_fog: Some(false),
            game_loop: None,
        },
    ));
    gameloop.send(req).await
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init_timed();

    let store = Arc::new(Surreal::new::<Ws>("localhost:8001").await?);

    store.use_ns("sc2bot").use_db("test").await?;
    log::info!("Game state connection initialized");

    store
        .query(
            queries::get("create_database")
                .await
                .with_context(|| "Expected create_database query to exist")?,
        )
        .await?
        .check()?;

    log::info!("SurrealDB tables defined");

    let mut gs = Bot::new(store).await;

    let mut sm = Core::init();

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

    let mut throughput_recorder = throughput::RollingRecorder::<16>::new();

    // wait for game to start
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Start the game loop
    request_observation(&mut gameloop).await?;
    while let Some(response) = gameloop.next().await {
        let loop_start = std::time::Instant::now();
        let Ok(response) = response else {
            let e = response.unwrap_err();
            log::error!("Error in game loop: {}", e);
            break;
        };

        // Process the response
        gs.update(response).await;

        // request next observation
        request_observation(&mut gameloop).await?;

        // record throughput
        throughput_recorder.record(
            std::time::Instant::now()
                .duration_since(loop_start)
                .as_millis() as f64,
        );
        let tp_ms = throughput_recorder.get_average();
        log::trace!(
            "Game loop iteration {idx}; throughput: {}ms/it | {} it/s",
            tp_ms,
            1_000.0 / tp_ms
        );
        idx += 1;
    }
    let _ended = gameloop.into_ended();

    log::info!("Game loop finished gracefully after {} iterations", idx);

    Ok(())
}
