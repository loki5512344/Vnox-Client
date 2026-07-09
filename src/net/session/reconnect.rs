use std::time::Duration;

/// Max delay between reconnect attempts.
pub const MAX_BACKOFF_SECS: u64 = 30;

/// Exponential backoff after a failed session (1s, 2s, 4s, … capped at 30s).
pub fn backoff_delay(failure_count: u32) -> Duration {
    let shift = failure_count.min(5);
    let secs = (1u64 << shift).min(MAX_BACKOFF_SECS);
    Duration::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_and_caps() {
        assert_eq!(backoff_delay(0).as_secs(), 1);
        assert_eq!(backoff_delay(1).as_secs(), 2);
        assert_eq!(backoff_delay(3).as_secs(), 8);
        assert_eq!(backoff_delay(10).as_secs(), MAX_BACKOFF_SECS);
    }
}
