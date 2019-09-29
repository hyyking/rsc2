use rsc2_derive::WrapEnum;

pub struct A;
pub struct B;
pub struct C;
pub struct D;

pub mod wrapper {
    #[derive(super::WrapEnum)]
    pub enum HasToWork {
        First(super::A),
        Second(super::B),
        Third(super::C),
        Fourth(super::D),
    }
}

/*
#[derive(WrapEnum)]
struct ShouldFail;

#[derive(WrapEnum)]
enum ShouldFailEnumOne {
    First(),
}
*/
