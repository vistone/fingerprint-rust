//! Timing protection module

// Timing anomaly detection thresholds
/// Standard deviation threshold in milliseconds for detecting bot-like timing consistency
const CONSISTENCY_THRESHOLD_MS: f64 = 5.0;

/// Minimum interval in milliseconds for human interaction - faster intervals are suspicious
const MIN_HUMAN_INTERVAL_MS: u64 = 50;

/// Maximum number of identical timing intervals before considering it automated behavior
const MAX_IDENTICAL_INTERVALS: usize = 3;

/// Timestamp precision for hiding timing resolution (milliseconds)
const TIMESTAMP_PRECISION_MS: u64 = 100;

/// Timing protector for adding random delays and obfuscation
pub struct TimingProtector {
    min_delay_ms: u64,
    max_delay_ms: u64,
}

impl TimingProtector {
    /// Create new timing protector
    pub fn new(min_delay_ms: u64, max_delay_ms: u64) -> Self {
        TimingProtector {
            min_delay_ms,
            max_delay_ms,
        }
    }

    /// Add random delay to response
    pub async fn add_random_delay(&self) {
        use rand::Rng;
        let delay = rand::thread_rng().gen_range(self.min_delay_ms..=self.max_delay_ms);
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    }

    /// Hide timing resolution
    pub fn hide_timing_resolution(&self, timestamp: u64) -> u64 {
        // Round timestamp to reduce precision
        (timestamp / TIMESTAMP_PRECISION_MS) * TIMESTAMP_PRECISION_MS
    }

    /// Detect timing anomalies
    ///
    /// Analyzes a sequence of timestamps to detect suspicious timing patterns
    /// that may indicate automated or scripted behavior.
    ///
    /// # Detection methods
    /// - Checks for abnormally consistent intervals (bot-like behavior)
    /// - Detects intervals that are too fast for human interaction
    /// - Identifies suspicious regularity in timing patterns
    pub fn detect_timing_anomalies(&self, timestamps: &[u64]) -> bool {
        if timestamps.len() < 3 {
            // Not enough data to detect anomalies
            return false;
        }

        // Calculate intervals between consecutive timestamps
        let mut intervals: Vec<u64> = timestamps
            .windows(2)
            .map(|w| w[1].saturating_sub(w[0]))
            .collect();

        if intervals.is_empty() {
            return false;
        }

        // Check 1: Detect suspiciously consistent intervals (bot-like behavior)
        let mean = intervals.iter().sum::<u64>() as f64 / intervals.len() as f64;
        let variance: f64 = intervals
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / intervals.len() as f64;
        let std_dev = variance.sqrt();

        // If standard deviation is very low, intervals are too consistent
        if std_dev < CONSISTENCY_THRESHOLD_MS && mean > 0.0 {
            return true;
        }

        // Check 2: Detect intervals that are too fast (< MIN_HUMAN_INTERVAL_MS is suspicious)
        let fast_intervals = intervals
            .iter()
            .filter(|&&interval| interval < MIN_HUMAN_INTERVAL_MS && interval > 0)
            .count();

        if fast_intervals > intervals.len() / 2 {
            // More than half of intervals are suspiciously fast
            return true;
        }

        // Check 3: Detect exact interval repetitions (highly suspicious)
        intervals.sort_unstable();
        let mut repetition_count = 1;
        let mut max_repetitions = 1;

        for i in 1..intervals.len() {
            if intervals[i] == intervals[i - 1] && intervals[i] > 0 {
                repetition_count += 1;
                max_repetitions = max_repetitions.max(repetition_count);
            } else {
                repetition_count = 1;
            }
        }

        // If more than MAX_IDENTICAL_INTERVALS, likely automated
        if max_repetitions > MAX_IDENTICAL_INTERVALS {
            return true;
        }

        false
    }
}
