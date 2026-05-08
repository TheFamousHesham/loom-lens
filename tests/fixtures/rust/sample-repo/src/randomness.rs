// Random-effect patterns.

use rand::Rng;
use uuid::Uuid;

pub fn roll_die() -> u8 {
    // expect: Random=definite
    rand::thread_rng().gen_range(1..=6)
}

pub fn new_session_id() -> Uuid {
    // expect: Random=definite
    Uuid::new_v4()
}

pub fn shuffle_in_place<T>(items: &mut [T]) {
    // expect: Random=definite, Mut=definite
    use rand::seq::SliceRandom;
    items.shuffle(&mut rand::thread_rng());
}

pub fn fixed_v5(name: &str) -> Uuid {
    // expect: (none — Pure; v5 is deterministic)
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes())
}
