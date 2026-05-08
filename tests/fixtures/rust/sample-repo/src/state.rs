// Mutation-effect patterns.

use std::sync::Mutex;

pub static ERRORS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn record_error(msg: &str) {
    // expect: Mut=definite
    let mut guard = ERRORS.lock().expect("poisoned");
    guard.push(msg.to_string());
}

pub fn append_in_place(items: &mut Vec<i32>, item: i32) {
    // expect: Mut=definite
    items.push(item);
}

pub fn clear_in_place<T>(items: &mut Vec<T>) {
    // expect: Mut=definite
    items.clear();
}

pub struct Counter {
    pub value: i64,
}

impl Counter {
    pub fn new() -> Self {
        // expect: (none — Pure)
        Self { value: 0 }
    }

    pub fn increment(&mut self) {
        // expect: Mut=definite
        self.value += 1;
    }
}

pub fn add_one(x: i64) -> i64 {
    // expect: Mut=possible
    // Pure body; name starts with "add".
    x + 1
}

pub fn with_item<T: Clone>(items: &[T], item: T) -> Vec<T> {
    // expect: (none — Pure)
    let mut out = items.to_vec();
    out.push(item);
    out
}
