extern crate prost;
extern crate websocket;

mod default;
mod result;

pub mod prelude;

pub mod sc2_api {
    include!(concat!(
        env!("OUT_DIR", "Couldn't find the generated rust-protobuf code"),
        "/sc2api_protocol.rs"
    ));
}
