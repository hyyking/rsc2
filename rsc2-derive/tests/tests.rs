mod expand;
mod try_into;

#[test]
fn test_enum() {
    use expand::*;
    match (A {}.into()) {
        wrapper::HasToWork::First(_) => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn test_try_into() {
    use std::convert::TryInto;
    use try_into::*;
    match 1_i32.try_into() {
        Ok(Value::One) => assert!(true),
        _ => assert!(false),
    }
}
