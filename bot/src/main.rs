mod throughput;

use std::{io, sync::Arc, time::Duration};

use chrono::Utc;
use futures::{SinkExt, StreamExt};
use log::info;
use rsc2::{
    prelude::{Difficulty, Player, Race, create_game},
    protocol,
    state_machine::Core,
};
use surrealdb::{
    RecordId, Surreal, Value,
    engine::remote::ws::{Client, Ws},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    id: RecordId,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HasPosition {
    #[serde(rename = "in")]
    unit: RecordId,
    #[serde(rename = "out")]
    position: RecordId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: RecordId,
    pub unit_type: u32,
}

struct GameState {
    start: Option<chrono::DateTime<Utc>>,
    store: Arc<Surreal<Client>>,
}

impl GameState {
    async fn new() -> Self {
        let store = Surreal::new::<Ws>("localhost:8001")
            .await
            .expect("Failed to connect to SurrealDB");
        store
            .use_ns("sc2bot")
            .use_db("test")
            .await
            .expect("Failed to use namespace and database");
        let store = Arc::new(store);
        Self { store, start: None }
    }

    async fn update(&mut self, response: &protocol::Response) {
        let now = Utc::now();
        if self.start.is_none() {
            // Initialize start time on first observation
            self.start = Some(now);
        }

        let protocol::Response {
            response: Some(protocol::response::Response::Observation(obs)),
            ..
        } = response
        else {
            log::trace!("Received non-observation response");
            return;
        };

        let protocol::ObservationRaw { units, .. } = obs
            .observation
            .as_ref()
            .and_then(|obs| obs.raw_data.as_ref())
            .unwrap();

        let (unit_data, (position_data, has_position)): (
            Vec<Unit>,
            (Vec<Position>, Vec<HasPosition>),
        ) = units
            .into_iter()
            .map(|unit| {
                let unit_id = RecordId::from(("unit", unit.tag() as i64));
                let unit_type = unit.unit_type();
                let unit_data = Unit {
                    id: unit_id.clone(),
                    unit_type,
                };

                let position_id = RecordId::from((
                    "position",
                    vec![
                        Value::from(unit_id.clone()),
                        <Value as std::str::FromStr>::from_str(&format!(
                            "{}",
                            surrealdb::Datetime::from(now)
                        ))
                        .expect("Failed to create position ID from datetime"),
                    ],
                ));
                let position_data = Position {
                    id: position_id.clone(),
                    x: unit.pos.as_ref().and_then(|p| p.x).unwrap_or(0.0),
                    y: unit.pos.as_ref().and_then(|p| p.y).unwrap_or(0.0),
                    z: unit.pos.as_ref().and_then(|p| p.z).unwrap_or(0.0),
                };

                let has_position = HasPosition {
                    unit: unit_id,
                    position: position_id,
                };
                (unit_data, (position_data, has_position))
            })
            .unzip();

        let response = self.store
            .query("BEGIN")
            .query("INSERT INTO unit $upsert_unit ON DUPLICATE KEY UPDATE last_seen = time::now();")
            .query("INSERT INTO position $upsert_position ON DUPLICATE KEY UPDATE x = $x, y = $y, z = $z;")
            .query("INSERT RELATION INTO has_position $upsert_has_position;")
            .query("COMMIT")
            .bind(("upsert_unit", unit_data))
            .bind(("upsert_position", position_data))
            .bind(("upsert_has_position", has_position))
            .await
            .expect("Failed to define unit table in SurrealDB");

        log::trace!(
            "Observation insertion status: {:?}",
            response.check().map(|_| "OK").unwrap_or("XX")
        );
    }

    async fn on_step(&mut self) -> Vec<protocol::Action> {
        vec![]
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
async fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();

    let mut gs = GameState::new().await;
    log::info!("Game state connection initialized");

    let response = gs
        .store
        .query("DEFINE TABLE OVERWRITE unit SCHEMALESS;")
        .query("DEFINE TABLE OVERWRITE position SCHEMALESS;")
        .query("DEFINE TABLE OVERWRITE has_position TYPE RELATION IN unit OUT position SCHEMALESS")
        .await
        .unwrap();

    response.check().expect("Failed to define SurrealDB tables");

    log::info!("SurrealDB tables defined");

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
        gs.update(&response).await;

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
