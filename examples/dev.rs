use rsc2::builder::MockGame;
use rsc2::Coordinator;

fn main() -> std::io::Result<()> {
    let mut c = Coordinator::new();
    let requests = c.run(MockGame::new())?;
    println!("requests {:?}", requests);
    Ok(())
}
