mod model;
use std::sync::Arc;

use chrono::{DateTime, Utc};
pub use model::{HasPosition, Position, Unit};

use rsc2::protocol;
use surrealdb::{RecordId, Surreal, Value, engine::remote::ws::Client};

pub struct World {
    store: Arc<Surreal<Client>>,
    start: Option<DateTime<Utc>>,
}

trait ValueExt {
    fn from_dt(dt: DateTime<Utc>) -> Value {
        <Value as std::str::FromStr>::from_str(&format!("{}", surrealdb::Datetime::from(dt)))
            .expect("passing surreal datetime that should be valid")
    }
}

impl ValueExt for Value {}

impl World {
    pub fn new(store: Arc<Surreal<Client>>) -> Self {
        Self { store, start: None }
    }

    pub async fn register_observation_raw(
        &mut self,
        observation: protocol::ObservationRaw,
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        if self.start.is_none() {
            // Initialize start time on first observation
            self.start = Some(now);
        }

        let protocol::ObservationRaw { units, .. } = observation;

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
                    vec![Value::from(unit_id.clone()), Value::from_dt(now)],
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
            .await?;

        log::trace!(
            "Observation insertion status: {:?}",
            response.check().map(|_| "OK").unwrap_or("XX")
        );
        Ok(())
    }
}
