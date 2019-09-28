mod expand;

#[test]
fn test_enum() {
    use expand::*;
    match (A {}.into()) {
        wrapper::HasToWork::First(_) => assert!(true),
        _ => assert!(false),
    }
}
