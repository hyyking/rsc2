use std::{io, sync::Arc, time::Duration};

use futures::{SinkExt, StreamExt};
use log::info;
use rsc2::{
    prelude::{Difficulty, Player, Race, create_game},
    protocol,
    state_machine::Core,
};
use surrealdb::{
    RecordId, Surreal,
    engine::remote::ws::{Client, Ws},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: RecordId,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub as_of: time::OffsetDateTime,
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
        Self { store }
    }

    async fn update(&mut self, response: &protocol::Response) {
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

                let position_id = RecordId::from(("position", surrealdb::Uuid::new_v4()));
                let position_data = Position {
                    id: position_id.clone(),
                    x: unit.pos.as_ref().and_then(|p| p.x).unwrap_or(0.0),
                    y: unit.pos.as_ref().and_then(|p| p.y).unwrap_or(0.0),
                    z: unit.pos.as_ref().and_then(|p| p.z).unwrap_or(0.0),
                    as_of: time::OffsetDateTime::now_utc(),
                };

                let has_position = HasPosition {
                    unit: unit_id.clone(),
                    position: position_id.clone(),
                };
                (unit_data, (position_data, has_position))
            })
            .unzip();

        let unitset = tokio::task::JoinSet::from_iter(unit_data.into_iter().map(|unit| {
            let store = Arc::clone(&self.store);
            tokio::task::spawn(async move {
                let _: Option<Unit> = store.upsert(unit.id.clone()).content(unit).await.unwrap();
            })
        }));

        let _: Vec<Position> = self
            .store
            .insert("position")
            .content(position_data)
            .await
            .expect("Failed to upsert positions in SurrealDB");

        let _: Vec<HasPosition> = self
            .store
            .insert("has_position")
            .relation(has_position)
            .await
            .expect("Failed to upsert HasPosition in SurrealDB");

        unitset.join_all().await;
    }

    async fn on_step(&mut self) -> Vec<protocol::Action> {
        vec![]
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    pretty_env_logger::init_timed();

    let mut gs = GameState::new().await;
    log::info!("Game state connection initialized");

    gs.store
        .query("DEFINE TABLE unit SCHEMALESS;")
        .await
        .unwrap();

    gs.store
        .query("DEFINE TABLE position SCHEMALESS;")
        .await
        .unwrap();

    gs.store
        .query("DEFINE TABLE has_position TYPE RELATION IN unit OUT position SCHEMALESS")
        .await
        .unwrap();

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

    let _state = loop {
        log::trace!("Game loop iteration {idx}");

        if idx == 0 {
            // wait for game to start
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;

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

        gs.update(&response).await;

        idx += 1;
    }?;
    log::info!("Game loop finished gracefully after {} iterations", idx);

    Ok(())
}
