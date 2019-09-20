extern crate tokio;

mod state_machine;

use tokio::net::TcpListener;
use tokio::prelude::*;

use state_machine::*;

fn main() {
    let sm = ProtocolStateMachine::default();
    let _nsm = ProtocolStateMachine::<InGame>::from(sm);

    let addr = "127.0.0.1:5000".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");
    let server = listener
        .incoming()
        .and_then(|socket| tokio::io::read_to_end(socket, vec![]))
        .map_err(|e| eprintln!("{:?}", e))
        .for_each(|(_, buff)| {
            println!("{:?}", String::from_utf8(buff));
            Ok(())
        });
    tokio::run(server)
}
