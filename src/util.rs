use std::time::{SystemTime, UNIX_EPOCH};

pub fn random_number() -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    nanos / 1000
}
