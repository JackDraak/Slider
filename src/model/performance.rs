use std::time::{Duration, Instant};

/// Performance metrics for entropy calculations
#[derive(Debug, Clone, Copy, Default)]
pub struct PerformanceMetrics {
    pub manhattan_time_micros: u64,
    pub heuristic_time_micros: u64,
    pub actual_time_micros: u64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Format duration in human-readable form
    pub fn format_duration(micros: u64) -> String {
        if micros == 0 {
            return "N/A".to_string();
        }
        if micros < 1000 {
            format!("{}μs", micros)
        } else if micros < 1_000_000 {
            format!("{:.2}ms", micros as f64 / 1000.0)
        } else {
            format!("{:.2}s", micros as f64 / 1_000_000.0)
        }
    }
}

/// Timer wrapper for measuring calculation performance
pub struct PerformanceTimer {
    start: Instant,
}

impl PerformanceTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_micros(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::start();
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_micros();

        // Should be at least 10ms (10,000 microseconds)
        assert!(elapsed >= 10_000);
    }

    #[test]
    fn test_format_duration_microseconds() {
        let formatted = PerformanceMetrics::format_duration(500);
        assert_eq!(formatted, "500μs");
    }

    #[test]
    fn test_format_duration_milliseconds() {
        let formatted = PerformanceMetrics::format_duration(5_000);
        assert_eq!(formatted, "5.00ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        let formatted = PerformanceMetrics::format_duration(2_500_000);
        assert_eq!(formatted, "2.50s");
    }

    #[test]
    fn test_format_duration_na() {
        let formatted = PerformanceMetrics::format_duration(0);
        assert_eq!(formatted, "N/A");
    }

    #[test]
    fn test_default_metrics() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.manhattan_time_micros, 0);
        assert_eq!(metrics.heuristic_time_micros, 0);
        assert_eq!(metrics.actual_time_micros, 0);
    }
}