use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: RecordId,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HasPosition {
    #[serde(rename = "in")]
    pub unit: RecordId,
    #[serde(rename = "out")]
    pub position: RecordId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: RecordId,
    pub unit_type: u32,
}
