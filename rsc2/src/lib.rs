#[macro_use]
extern crate log;

mod ingame;

pub mod prelude;
pub mod state_machine;
pub use rsc2_pb::protocol;

use std::io;

use rsc2_pb::codec::S2Codec;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub type Connection = Framed<TcpStream, S2Codec>;

pub async fn connect(addr: impl std::net::ToSocketAddrs) -> io::Result<Connection> {
    use tokio_util::codec::FramedParts;
    use websocket_lite::ClientBuilder;

    let addr = format!(
        "ws://{}/sc2api",
        addr.to_socket_addrs()
            .ok()
            .as_mut()
            .and_then(Iterator::next)
            .unwrap()
    );
    let client = ClientBuilder::new(&addr).map(ClientBuilder::async_connect_insecure);
    match client {
        Ok(client) => match client.await {
            Ok(framed) => {
                let FramedParts { io, codec, .. } = framed.into_parts();
                Ok(Framed::from_parts(FramedParts::new::<
                    rsc2_pb::protocol::Request,
                >(
                    io, S2Codec::from(codec)
                )))
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::ConnectionRefused, e)),
        },
        Err(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid address",
        )),
    }
}
