use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn verify_timestamp_10_minute_window(timestamp: u64, window: Duration) -> bool {
    let timestamp: Duration = Duration::from_secs(timestamp);
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    timestamp > (since_the_epoch - window)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use crate::mpc::utils::check_timestamp::verify_timestamp_10_minute_window;

    const TEN_MINUTES: Duration = Duration::from_secs(600);

    #[test]
    fn check_that_now_is_true() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        assert!(verify_timestamp_10_minute_window(now.as_secs(), TEN_MINUTES));
    }

    #[test]
    fn check_too_old() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        assert!(
            !verify_timestamp_10_minute_window((now - Duration::from_secs(1000)).as_secs(),
                                                   TEN_MINUTES
            )
        );
    }
}