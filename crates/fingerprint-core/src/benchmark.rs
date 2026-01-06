//! Performance benchmarking utilities for fingerprint-rust
//! 
//! This module provides comprehensive benchmarking tools to measure
//! and track performance across different components.

use std::time::{Duration, Instant};

/// Performance metrics for HTTP operations
#[derive(Debug, Clone)]
pub struct HttpMetrics {
    /// Total request duration
    pub total_duration: Duration,
    /// DNS resolution time
    pub dns_time: Option<Duration>,
    /// TCP connection time
    pub tcp_connect_time: Option<Duration>,
    /// TLS handshake time
    pub tls_handshake_time: Option<Duration>,
    /// Time to first byte
    pub ttfb: Option<Duration>,
    /// Response download time
    pub download_time: Option<Duration>,
    /// Request size in bytes
    pub request_size: usize,
    /// Response size in bytes
    pub response_size: usize,
}

impl HttpMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            total_duration: Duration::from_secs(0),
            dns_time: None,
            tcp_connect_time: None,
            tls_handshake_time: None,
            ttfb: None,
            download_time: None,
            request_size: 0,
            response_size: 0,
        }
    }

    /// Calculate throughput in bytes per second
    pub fn throughput(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.response_size as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Calculate requests per second (if known)
    pub fn requests_per_second(&self, num_requests: usize) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            num_requests as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }
}

impl Default for HttpMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark runner for HTTP operations
pub struct Benchmark {
    name: String,
    iterations: usize,
    metrics: Vec<HttpMetrics>,
}

impl Benchmark {
    /// Create a new benchmark with the given name
    pub fn new(name: impl Into<String>, iterations: usize) -> Self {
        Self {
            name: name.into(),
            iterations,
            metrics: Vec::with_capacity(iterations),
        }
    }

    /// Run the benchmark with the provided function
    pub fn run<F>(&mut self, mut f: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut() -> Result<HttpMetrics, Box<dyn std::error::Error>>,
    {
        for i in 0..self.iterations {
            let metrics = f()?;
            self.metrics.push(metrics);
            
            if (i + 1) % 10 == 0 {
                println!("[{}] Completed {}/{} iterations", self.name, i + 1, self.iterations);
            }
        }
        Ok(())
    }

    /// Calculate and display statistics
    pub fn report(&self) {
        if self.metrics.is_empty() {
            println!("[{}] No metrics collected", self.name);
            return;
        }

        let total_times: Vec<f64> = self.metrics
            .iter()
            .map(|m| m.total_duration.as_secs_f64() * 1000.0)
            .collect();

        let avg = total_times.iter().sum::<f64>() / total_times.len() as f64;
        let min = total_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = total_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate standard deviation
        let variance = total_times
            .iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f64>() / total_times.len() as f64;
        let std_dev = variance.sqrt();

        // Calculate percentiles
        let mut sorted_times = total_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = sorted_times[sorted_times.len() / 2];
        let p95 = sorted_times[sorted_times.len() * 95 / 100];
        let p99 = sorted_times[sorted_times.len() * 99 / 100];

        // Calculate total throughput
        let total_bytes: usize = self.metrics.iter().map(|m| m.response_size).sum();
        let total_time: f64 = self.metrics.iter().map(|m| m.total_duration.as_secs_f64()).sum();
        let avg_throughput = if total_time > 0.0 {
            total_bytes as f64 / total_time / 1024.0 / 1024.0 // MB/s
        } else {
            0.0
        };

        println!("\n========== Benchmark: {} ==========", self.name);
        println!("Iterations: {}", self.iterations);
        println!("\nLatency (ms):");
        println!("  Average:  {:.2}", avg);
        println!("  Min:      {:.2}", min);
        println!("  Max:      {:.2}", max);
        println!("  Std Dev:  {:.2}", std_dev);
        println!("  p50:      {:.2}", p50);
        println!("  p95:      {:.2}", p95);
        println!("  p99:      {:.2}", p99);
        println!("\nThroughput:");
        println!("  Average:  {:.2} MB/s", avg_throughput);
        println!("  Total:    {} bytes", total_bytes);
        println!("  Requests: {:.2} req/s", self.iterations as f64 / total_time);
        println!("=====================================\n");
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and return elapsed duration
    pub fn stop(self) -> Duration {
        self.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = HttpMetrics::new();
        assert_eq!(metrics.total_duration.as_secs(), 0);
        assert_eq!(metrics.request_size, 0);
        assert_eq!(metrics.response_size, 0);
    }

    #[test]
    fn test_throughput_calculation() {
        let mut metrics = HttpMetrics::new();
        metrics.response_size = 1000;
        metrics.total_duration = Duration::from_secs(1);
        assert_eq!(metrics.throughput(), 1000.0);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_benchmark_creation() {
        let bench = Benchmark::new("test", 10);
        assert_eq!(bench.name, "test");
        assert_eq!(bench.iterations, 10);
    }
}
