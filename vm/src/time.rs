use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn time_since_epoch() -> Duration {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
}

pub(crate) fn get_nano_time() -> i64 {
    time_since_epoch().as_nanos() as i64
}

pub(crate) fn get_current_time_millis() -> i64 {
    time_since_epoch().as_millis() as i64
}
