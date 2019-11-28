use std::io;

use crate::api;

use bytes::BytesMut;
use prost::Message;
use tokio_codec::{Decoder, Encoder, Framed};
use tokio_net::tcp::TcpStream;
use websocket_codec::{Message as WSMessage, MessageCodec};

/// Protobuf and TCP based [`Framed`](tokio::codec::Framed) type
pub type SC2ProtobufClient = Framed<TcpStream, SC2ProtobufCodec>;

/// wrap a Framed [`TcpStream`](tokio_net::tcp::TcpStream) with a [`MessageCodec`](websocket_codec::MessageCodec) into a [`SC2ProtobufClient`](SC2ProtobufClient)
pub fn from_framed(old: Framed<TcpStream, MessageCodec>) -> SC2ProtobufClient {
    let parts = old.into_parts();
    Framed::new(parts.io, parts.codec.into())
}

/// Codec for encoding an decoding websocket protobuf messages
#[allow(missing_debug_implementations)]
pub struct SC2ProtobufCodec {
    inner: MessageCodec,
}

impl From<MessageCodec> for SC2ProtobufCodec {
    fn from(inner: MessageCodec) -> Self {
        Self { inner }
    }
}

impl Decoder for SC2ProtobufCodec {
    type Item = api::Response;
    type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.inner.decode(src) {
            Ok(Some(message)) => Ok(Some(api::Response::decode(message.into_data())?)),
            Ok(None) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}

impl Encoder for SC2ProtobufCodec {
    type Item = api::Request;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut buffer = Vec::with_capacity(item.encoded_len());
        item.encode(&mut buffer)?;
        match self.inner.encode(WSMessage::binary(buffer), dst) {
            Ok(()) => Ok(()),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}
