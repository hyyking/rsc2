//! *rsc2-pb*
//!
//! This crate provides the raw elements used in [`rsc2`](rsc2)
//!
//! Currently implements:
//!
//! * Rust code generated from the protobuf descriptor.
//! * Custom traits and implementations for the generated code.
//! * Codec to be used alongside a websocket client (use [`websocket_codec`](websocket_codec)
//!   under the hood).
//!
//! # Features
//!
//! * `default`: `encoding` + `rsc2_macro`
//! * `encoding`: prost message derive on api structs to convert them to bytes
//! * `codec`: api protobuf encoding/decoding on a stream [dep: `encoding`]
//! * `rsc2_macro`: custom derives to reduce verbosity of the api

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

mod default;

#[cfg(feature = "codec")]
#[doc(inline)]
pub mod codec;

/// rsc2-pb prelude traits
pub mod prelude;

#[allow(missing_docs)] // TODO: Generate better documentation for this module
pub mod api {
    include!(concat!(
        env!("OUT_DIR", "Couldn't find the generated rust-protobuf code"),
        "/sc2api_protocol.rs"
    ));
}

pub use default::DefaultConfig;

/// validate a [`api::Status`](api::Status) variable and variant.
#[macro_export]
macro_rules! validate_status {
    ($status:expr => $variant:path) => {{
        use ::core::convert::TryFrom;
        let status: ::core::option::Option<i32> = $status;
        let _: ::rsc2_pb::api::Status = $variant;
        status
            .ok_or_else(|| {
                ::std::io::Error::new(
                    ::std::io::ErrorKind::ConnectionAborted,
                    "Missing Status Code",
                )
            })
            .and_then(
                |status| match ::rsc2_pb::api::Status::try_from(status).ok() {
                    Some($variant) => Ok($variant),
                    Some(e) => Err(::std::io::Error::new(
                        ::std::io::ErrorKind::ConnectionAborted,
                        format!(r#"Unexpected "{:?}""#, e),
                    )),
                    None => Err(::std::io::Error::new(
                        ::std::io::ErrorKind::ConnectionAborted,
                        "Wrong status Code",
                    )),
                },
            )
    }};
}
