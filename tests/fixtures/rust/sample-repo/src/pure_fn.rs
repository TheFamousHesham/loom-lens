// Pure functions — control case for the inference engine.
// None of these should be tagged with any effect.

pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

pub fn hypotenuse(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

pub fn join_words(words: &[&str]) -> String {
    words.join(" ")
}

pub fn reverse_pair<A, B>(pair: (A, B)) -> (B, A) {
    let (a, b) = pair;
    (b, a)
}

pub fn upper_first(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn fibonacci(n: u32) -> u64 {
    if n < 2 {
        return n as u64;
    }
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..(n - 1) {
        let t = a + b;
        a = b;
        b = t;
    }
    b
}
