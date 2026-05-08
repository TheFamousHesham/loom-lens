// Throw-effect patterns (Rust: panic + Result).
//
// Per ADR 0002 refinement: Result<T, E> returning is NOT an effect; only
// panic-paths are tagged Throw.

pub fn raise_if_invalid(id: u64) {
    // expect: Throw=definite
    if id == 0 {
        panic!("zero id is not allowed");
    }
}

pub fn unwrap_unsafely(s: &str) -> i32 {
    // expect: Throw=definite
    s.parse::<i32>().unwrap()
}

pub fn expect_unsafely(s: &str) -> i32 {
    // expect: Throw=definite
    s.parse::<i32>().expect("must be an integer")
}

pub fn index_directly(arr: &[i32], i: usize) -> i32 {
    // expect: Throw=definite
    arr[i]
}

pub fn definitely_unreachable() -> ! {
    // expect: Throw=definite
    unreachable!("this branch is provably dead")
}

pub fn fallible_no_panic(s: &str) -> Result<i32, std::num::ParseIntError> {
    // expect: (none — Pure; Result-returning is not an effect)
    s.parse::<i32>()
}

pub fn validate_name(name: &str) -> &str {
    // expect: Throw=possible
    name
}
