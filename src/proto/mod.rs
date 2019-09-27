mod default;
mod result;
mod wrap;

pub mod prelude;
pub mod sc2_api {
    include!(concat!(env!("OUT_DIR"), "/sc2api_protocol.rs"));
}
