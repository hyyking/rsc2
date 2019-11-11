use rsc2::Commands::{CreateGame, JoinGame, Launched};
use rsc2::Coordinator;

fn main() -> std::io::Result<()> {
    let mut c = Coordinator::new();
    c.run(&[Launched {}, CreateGame {}, JoinGame {}])
}
