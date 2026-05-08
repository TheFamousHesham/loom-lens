// Async-effect patterns.

use std::future::Future;

pub async fn fetch_all(urls: Vec<String>) -> Vec<String> {
    // expect: Async=definite
    let mut out = Vec::with_capacity(urls.len());
    for u in urls {
        out.push(fetch_one(u).await);
    }
    out
}

pub async fn fetch_one(url: String) -> String {
    // expect: Async=definite
    // (No actual await of an external future, but the function is `async fn`.)
    url
}

pub fn pending() -> impl Future<Output = i32> {
    // expect: Async=definite
    // Returns `impl Future<...>` — definite Async.
    async { 42 }
}

pub fn worker_loop() {
    // expect: Async=possible
    // Pure body; name pattern only.
}

pub async fn sleep_for_ms(ms: u64) {
    // expect: Async=definite, Time=definite
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}
