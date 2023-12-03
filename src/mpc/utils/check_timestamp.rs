use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn verify_timestamp_10_minute_window(timestamp: u64, window: Duration) -> bool {
    let timestamp: Duration = Duration::from_secs(timestamp);
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    timestamp >= (since_the_epoch - window) && timestamp <= (since_the_epoch + window)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use crate::mpc::utils::check_timestamp::verify_timestamp_10_minute_window;

    const FIVE_MINUTES: Duration = Duration::from_secs(300);

    fn get_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    #[test]
    fn check_that_now_is_true() {
        assert!(verify_timestamp_10_minute_window(get_now(), FIVE_MINUTES));
    }

    #[test]
    fn check_too_old() {
        let too_old = get_now() - 310;
        assert!(
            !verify_timestamp_10_minute_window(too_old, FIVE_MINUTES)
        );
    }

    #[test]
    fn check_too_in_the_future() {
        let too_new = get_now() + 310;

        assert!(
            !verify_timestamp_10_minute_window(too_new, FIVE_MINUTES)
        );
    }
}