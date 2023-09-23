use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TEN_MINUTES: Duration = Duration::from_secs(600);

pub fn verify_timestamp_10_minute_window(timestamp: u64) -> bool {
    let timestamp: Duration = Duration::from_secs(timestamp);
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    timestamp > (since_the_epoch - TEN_MINUTES)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use crate::utils::check_timestamp::verify_timestamp_10_minute_window;

    #[test]
    fn check_that_now_is_true() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        assert!(verify_timestamp_10_minute_window(now.as_secs()));
    }

    #[test]
    fn check_too_old() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        assert!(!verify_timestamp_10_minute_window((now - Duration::from_secs(1000)).as_secs()));
    }
}