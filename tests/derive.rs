#[test]
fn wrap_enum() {
    use rsc2_derive::WrapEnum;

    struct A;
    struct B;
    struct C;
    struct D;

    #[derive(WrapEnum)]
    enum HasToWork {
        First(A),
        Second(B),
        Third(C),
        Fourth(D),
    }

    match A.into() {
        HasToWork::First(_) => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn try_into() {
    use rsc2_derive::TryIntoEnum;
    use std::convert::TryInto;

    #[derive(TryIntoEnum)]
    pub enum Value {
        One = 1,
        Two = 2,
        Three = 3,
    }
    match 1_i32.try_into() {
        Ok(Value::One) => assert!(true),
        _ => assert!(false),
    }
}
