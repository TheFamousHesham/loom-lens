// Time-effect patterns.

use std::thread;
use std::time::{Duration, Instant, SystemTime};

pub fn now_instant() -> Instant {
    // expect: Time=definite
    Instant::now()
}

pub fn now_system() -> SystemTime {
    // expect: Time=definite
    SystemTime::now()
}

pub fn pause(secs: u64) {
    // expect: Time=definite
    thread::sleep(Duration::from_secs(secs));
}

pub fn current_time_label() -> &'static str {
    // expect: Time=possible
    "now"
}
