//! Automatic reconnection configuration
//!
//! Provides reconnection strategy with exponential backoff for resilient connections.

use std::time::Duration;

/// Reconnection strategy configuration
///
/// Used with [`ClientBuilder`](crate::io::builder::ClientBuilder) to enable automatic
/// reconnection after network failures.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::builder::ClientBuilder;
/// use openigtlink_rust::io::reconnect::ReconnectConfig;
///
/// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// // Auto-reconnect with default settings (10 attempts)
/// let config = ReconnectConfig::default();
/// let client = ClientBuilder::new()
///     .tcp("127.0.0.1:18944")
///     .async_mode()
///     .with_reconnect(config)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// ```no_run
/// use openigtlink_rust::io::builder::ClientBuilder;
/// use openigtlink_rust::io::reconnect::ReconnectConfig;
///
/// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// // Infinite retries
/// let config = ReconnectConfig::infinite();
/// let client = ClientBuilder::new()
///     .tcp("127.0.0.1:18944")
///     .async_mode()
///     .with_reconnect(config)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts (None = infinite)
    pub max_attempts: Option<usize>,
    /// Initial delay before first reconnection attempt
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
    /// Backoff multiplier (delay is multiplied by this after each attempt)
    pub backoff_multiplier: f64,
    /// Whether to add random jitter to delays
    pub use_jitter: bool,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: Some(10),
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl ReconnectConfig {
    /// Create config with infinite retries
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::reconnect::ReconnectConfig;
    ///
    /// let config = ReconnectConfig::infinite();
    /// assert_eq!(config.max_attempts, None);
    /// ```
    pub fn infinite() -> Self {
        Self {
            max_attempts: None,
            ..Default::default()
        }
    }

    /// Create config with specific max attempts
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::reconnect::ReconnectConfig;
    ///
    /// let config = ReconnectConfig::with_max_attempts(5);
    /// assert_eq!(config.max_attempts, Some(5));
    /// ```
    pub fn with_max_attempts(attempts: usize) -> Self {
        Self {
            max_attempts: Some(attempts),
            ..Default::default()
        }
    }

    /// Create config with custom delays
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::reconnect::ReconnectConfig;
    /// use std::time::Duration;
    ///
    /// let config = ReconnectConfig::with_delays(
    ///     Duration::from_millis(500),
    ///     Duration::from_secs(60)
    /// );
    /// assert_eq!(config.initial_delay, Duration::from_millis(500));
    /// assert_eq!(config.max_delay, Duration::from_secs(60));
    /// ```
    pub fn with_delays(initial: Duration, max: Duration) -> Self {
        Self {
            initial_delay: initial,
            max_delay: max,
            ..Default::default()
        }
    }

    /// Calculate delay for a given attempt number
    ///
    /// Uses exponential backoff with optional jitter.
    pub(crate) fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);

        let mut delay = Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f64) as u64);

        // Add jitter if enabled (0-25% random variation)
        if self.use_jitter {
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hash, Hasher};

            let mut hasher = RandomState::new().build_hasher();
            attempt.hash(&mut hasher);
            let hash = hasher.finish();
            let jitter = (hash % 25) as f64 / 100.0; // 0-25%

            let jitter_ms = (delay.as_millis() as f64 * jitter) as u64;
            delay = Duration::from_millis(delay.as_millis() as u64 + jitter_ms);
        }

        delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_config_defaults() {
        let config = ReconnectConfig::default();
        assert_eq!(config.max_attempts, Some(10));
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }

    #[test]
    fn test_reconnect_config_infinite() {
        let config = ReconnectConfig::infinite();
        assert_eq!(config.max_attempts, None);
    }

    #[test]
    fn test_reconnect_config_delay_calculation() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            use_jitter: false,
            max_attempts: Some(10),
        };

        // Test exponential backoff
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
    }
}
