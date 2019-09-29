use rsc2_derive::TryIntoEnum;

#[derive(TryIntoEnum)]
pub enum Value {
    One = 1,
    Two = 2,
    Three = 3,
}
