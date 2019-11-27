mod default;

#[cfg(feature = "codec")]
pub mod codec;

pub mod prelude;

pub mod api {
    include!(concat!(
        env!("OUT_DIR", "Couldn't find the generated rust-protobuf code"),
        "/sc2api_protocol.rs"
    ));
}

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
