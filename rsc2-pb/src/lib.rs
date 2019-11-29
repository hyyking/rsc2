//! This crate provides the raw elements used in [`rsc2`](rsc2)
//!
//! Currently implements:
//!
//! * Rust code generated from the protobuf api.
//! * Custom traits and implementations for the generated code.
//! * Codec to be used alongside a websocket client (uses [`websocket_codec`](crate::websocket_codec)
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

/// validate a [`Status`](crate::api::Status) variable and variant.
///
/// This macro returns a [`io::Result<Status>`](std::io::Result) block.
///
/// # Example
///
/// ```no_run
/// use rsc2_pb::{validate_status, api};
///
/// fn main() {
///     // This what the status will look like in a response.
///     let after_start = Some(api::Status::InGame as i32);
///     assert!(validate_status!(after_start => api::Status::InGame).is_ok());
///     assert!(validate_status!(after_start => api::Status::Ended).is_err());
///
///     let wrong = Some(99999);
///     assert!(validate_status!(after_start => api::Status::InGame).is_err());
///
///     let empty = None;
///     assert!(validate_status!(empty => api::Status::Ended).is_err());
///
/// }
/// ```
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
                        "Status code is empty",
                    )),
                },
            )
    }};
}
