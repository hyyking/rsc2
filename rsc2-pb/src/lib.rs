//! This crate provides the raw elements used in [`rsc2`](rsc2)
//!
//! Currently implements:
//!
//! * Rust code generated from the protobuf api.
//! * Codec to be used alongside a websocket client (uses [`websocket_codec`](crate::websocket_codec)
//!   under the hood).
//!
//! # Features
//!
//! * `codec`: api protobuf encoding/decoding on a stream [dep: `encoding`]

#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Generated protobuf protocol
#[allow(missing_docs)] // TODO: add custom documention for protocol
pub mod protocol {
    include!(concat!(
        env!("OUT_DIR", "Couldn't find the generated rust-protobuf code"),
        "/sc2api_protocol.rs"
    ));
}

#[cfg(feature = "codec")]
#[cfg_attr(docsrs, doc(cfg(feature = "codec")))]
pub mod codec {
    //! A `tokio_util::codec` for sc2api websocket protobuf messages
    use std::{fmt, io};

    use crate::protocol;

    use bytes::BytesMut;
    use prost::Message as _;
    use tokio_util::codec::{Decoder, Encoder};
    use websocket_codec::{Message, MessageCodec};

    /// Client codec to interact with a SC2 instance
    ///
    /// This instance keeps track of the request id, custom ids will be overritten.
    pub struct S2Codec {
        id: u32,
        inner: MessageCodec,
    }

    impl S2Codec {
        /// Returns current request id
        pub fn id(&self) -> u32 {
            self.id
        }
        /// References the underlying websocket codec
        pub fn message_codec(&mut self) -> &mut MessageCodec {
            &mut self.inner
        }
    }

    impl fmt::Debug for S2Codec {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Codec").field("id", &self.id).finish()
        }
    }

    impl From<MessageCodec> for S2Codec {
        fn from(inner: MessageCodec) -> Self {
            Self { id: 0, inner }
        }
    }

    impl Decoder for S2Codec {
        type Item = protocol::Response;
        type Error = io::Error;
        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            match self.inner.decode(src) {
                Ok(Some(message)) => Ok(Some(protocol::Response::decode(message.into_data())?)),
                Ok(None) => Ok(None),
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            }
        }
    }

    impl Encoder<protocol::Request> for S2Codec {
        type Error = io::Error;
        fn encode(
            &mut self,
            item: protocol::Request,
            dst: &mut BytesMut,
        ) -> Result<(), Self::Error> {
            self.id = item.id();
            let mut buffer = BytesMut::with_capacity(item.encoded_len());
            item.encode(&mut buffer)?;
            match self.inner.encode(Message::binary(buffer), dst) {
                Ok(()) => Ok(()),
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            }
        }
    }
    macro_rules! impl_req_encoder {
        {$($variant:ident => $request:ident),+} => {
            $(
            impl Encoder<$crate::protocol::$request> for S2Codec {
                type Error = io::Error;
                fn encode(
                    &mut self,
                    item: $crate::protocol::$request,
                    dst: &mut ::bytes::BytesMut,
                ) -> Result<(), Self::Error> {
                    self.id = self.id.wrapping_add(1);
                    let request = $crate::protocol::Request {
                        id: Some(self.id),
                        request: Some(
                            $crate::protocol::request::Request::$variant(item),
                        ),
                    };
                    let mut buffer = ::bytes::BytesMut::with_capacity(request.encoded_len());
                    request.encode(&mut buffer)?;
                    match self.inner.encode(Message::binary(buffer), dst) {
                        Ok(()) => Ok(()),
                        Err(e) => Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData, e)),
                    }
                }
            }
            )*
        };
    }
    impl_req_encoder! {
        CreateGame => RequestCreateGame,
        JoinGame => RequestJoinGame,
        RestartGame => RequestRestartGame,
        StartReplay => RequestStartReplay,
        LeaveGame => RequestLeaveGame,
        QuickSave => RequestQuickSave,
        QuickLoad => RequestQuickLoad,
        Quit => RequestQuit,
        GameInfo => RequestGameInfo,
        Observation => RequestObservation,
        Action => RequestAction,
        ObsAction => RequestObserverAction,
        Step => RequestStep,
        Data => RequestData,
        Query => RequestQuery,
        SaveReplay => RequestSaveReplay,
        MapCommand => RequestMapCommand,
        ReplayInfo => RequestReplayInfo,
        AvailableMaps => RequestAvailableMaps,
        SaveMap => RequestSaveMap,
        Ping => RequestPing,
        Debug => RequestDebug
    }
}
