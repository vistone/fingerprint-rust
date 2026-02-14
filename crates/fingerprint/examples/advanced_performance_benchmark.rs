//! Advanced Performance Benchmarking Example
//!
//! This example demonstrates comprehensive performance testing and optimization
//! techniques for the fingerprint-rust library. It showcases:
//!
//! - High-precision timing with multiple iterations
//! - Statistical analysis (mean, median, stddev, percentiles)
//! - Throughput calculations
//! - Memory allocation profiling
//! - Comparative analysis between different approaches
//! - Performance regression detection
//!
//! Based on industry best practices from Google Benchmark, Criterion.rs,
//! and performance engineering guidelines.

use fingerprint::profiles::{chrome_133, firefox_133};
use fingerprint::{HttpClient, HttpClientConfig};
use std::time::{Duration, Instant};

/// Statistical results from benchmark runs
#[derive(Debug, Clone)]
struct BenchmarkStats {
    mean: Duration,
    median: Duration,
    std_dev: Duration,
    min: Duration,
    max: Duration,
    p95: Duration,
    p99: Duration,
    throughput: f64, // operations per second
}

impl BenchmarkStats {
    fn from_samples(mut samples: Vec<Duration>) -> Self {
        samples.sort();
        let count = samples.len() as f64;

        let mean_nanos: f64 = samples.iter().map(|d| d.as_nanos() as f64).sum::<f64>() / count;
        let mean = Duration::from_nanos(mean_nanos as u64);

        let median = samples[samples.len() / 2];
        let min = samples[0];
        let max = samples[samples.len() - 1];

        let p95_idx = (samples.len() as f64 * 0.95) as usize;
        let p99_idx = (samples.len() as f64 * 0.99) as usize;
        let p95 = samples[p95_idx.min(samples.len() - 1)];
        let p99 = samples[p99_idx.min(samples.len() - 1)];

        // Calculate standard deviation
        let variance: f64 = samples
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>()
            / count;
        let std_dev = Duration::from_nanos(variance.sqrt() as u64);

        let throughput = 1_000_000_000.0 / mean_nanos; // ops/sec

        Self {
            mean,
            median,
            std_dev,
            min,
            max,
            p95,
            p99,
            throughput,
        }
    }

    fn report(&self, name: &str) {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║ Benchmark: {:<50} ║", name);
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║ Mean:       {:>12.3} ms  (± {:.3} ms)              ║",
            self.mean.as_secs_f64() * 1000.0,
            self.std_dev.as_secs_f64() * 1000.0
        );
        println!(
            "║ Median:     {:>12.3} ms                             ║",
            self.median.as_secs_f64() * 1000.0
        );
        println!(
            "║ Min:        {:>12.3} ms                             ║",
            self.min.as_secs_f64() * 1000.0
        );
        println!(
            "║ Max:        {:>12.3} ms                             ║",
            self.max.as_secs_f64() * 1000.0
        );
        println!(
            "║ P95:        {:>12.3} ms                             ║",
            self.p95.as_secs_f64() * 1000.0
        );
        println!(
            "║ P99:        {:>12.3} ms                             ║",
            self.p99.as_secs_f64() * 1000.0
        );
        println!(
            "║ Throughput: {:>12.2} ops/sec                       ║",
            self.throughput
        );
        println!("╚══════════════════════════════════════════════════════════════╝");
    }
}

/// Run a benchmark with specified iterations
fn benchmark<F>(name: &str, iterations: usize, mut operation: F) -> BenchmarkStats
where
    F: FnMut() -> Result<(), Box<dyn std::error::Error>>,
{
    println!(
        "\n⏱️  Running benchmark: {} ({} iterations)...",
        name, iterations
    );

    // Warm-up phase (10% of iterations)
    let warmup = iterations / 10;
    print!("   Warming up ({} iterations)... ", warmup);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    for _ in 0..warmup {
        let _ = operation();
    }
    println!("✓");

    // Actual measurement phase
    print!("   Measuring... ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut samples = Vec::with_capacity(iterations);
    for i in 0..iterations {
        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        if result.is_err() {
            println!("\n   ⚠️  Warning: operation failed at iteration {}", i);
        }
        samples.push(elapsed);
    }
    println!("✓");

    BenchmarkStats::from_samples(samples)
}

/// Benchmark fingerprint generation performance
fn bench_fingerprint_generation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("               FINGERPRINT GENERATION BENCHMARKS");
    println!("═══════════════════════════════════════════════════════════════");

    // Benchmark Chrome 133 fingerprint generation
    let stats = benchmark("Chrome 133 Fingerprint Generation", 10_000, || {
        let _profile = chrome_133();
        Ok(())
    });
    stats.report("Chrome 133 Fingerprint Generation");

    // Benchmark Firefox 133 fingerprint generation
    let stats = benchmark("Firefox 133 Fingerprint Generation", 10_000, || {
        let _profile = firefox_133();
        Ok(())
    });
    stats.report("Firefox 133 Fingerprint Generation");

    // Benchmark TLS ClientHello spec generation
    let stats = benchmark("TLS ClientHello Spec Generation", 1_000, || {
        let profile = chrome_133();
        let _spec = &profile.tls_config;
        Ok(())
    });
    stats.report("TLS ClientHello Spec Generation");

    Ok(())
}

/// Benchmark HTTP client configuration
fn bench_http_client_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                 HTTP CLIENT CONFIG BENCHMARKS");
    println!("═══════════════════════════════════════════════════════════════");

    let stats = benchmark("HTTP Client Creation", 10_000, || {
        let config = HttpClientConfig {
            user_agent: "Mozilla/5.0".to_string(),
            prefer_http2: true,
            ..Default::default()
        };
        let _client = HttpClient::new(config);
        Ok(())
    });
    stats.report("HTTP Client Creation");

    Ok(())
}

/// Comparative benchmark between different approaches
fn bench_comparative_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                  COMPARATIVE ANALYSIS");
    println!("═══════════════════════════════════════════════════════════════");

    let iterations = 1_000;

    // Approach 1: Creating new profile each time
    let stats1 = benchmark("Approach 1: New Profile Each Time", iterations, || {
        let profile = chrome_133();
        let _spec = &profile.tls_config;
        Ok(())
    });

    // Approach 2: Reusing profile
    let profile = chrome_133();
    let stats2 = benchmark("Approach 2: Reusing Profile", iterations, || {
        let _spec = &profile.tls_config;
        Ok(())
    });

    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│                    COMPARISON RESULTS                      │");
    println!("├────────────────────────────────────────────────────────────┤");
    println!(
        "│ Approach 1 (New): {:.3} ms mean                       │",
        stats1.mean.as_secs_f64() * 1000.0
    );
    println!(
        "│ Approach 2 (Reuse): {:.3} ms mean                     │",
        stats2.mean.as_secs_f64() * 1000.0
    );
    let speedup = stats1.mean.as_secs_f64() / stats2.mean.as_secs_f64();
    println!(
        "│ Speedup: {:.2}x                                         │",
        speedup
    );
    println!("└────────────────────────────────────────────────────────────┘");

    Ok(())
}

/// Memory allocation profiling simulation
fn bench_memory_profile() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                  MEMORY PROFILING (Simulated)");
    println!("═══════════════════════════════════════════════════════════════");

    println!("\n💡 Note: For detailed memory profiling, use:");
    println!("   - valgrind --tool=massif");
    println!("   - heaptrack");
    println!("   - cargo-instruments (on macOS)");
    println!("   - perf record -g (on Linux)");

    // Estimate memory usage
    let profile = chrome_133();
    let spec = &profile.tls_config;

    let estimated_profile_size = std::mem::size_of_val(&profile);
    let estimated_spec_size = std::mem::size_of_val(spec);

    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Estimated Memory Usage (Stack only, approximate)          │");
    println!("├────────────────────────────────────────────────────────────┤");
    println!(
        "│ Profile struct:     {:>6} bytes                         │",
        estimated_profile_size
    );
    println!(
        "│ ClientHello spec:   {:>6} bytes                         │",
        estimated_spec_size
    );
    println!("└────────────────────────────────────────────────────────────┘");

    Ok(())
}

/// Performance regression detection
fn bench_regression_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("              PERFORMANCE REGRESSION DETECTION");
    println!("═══════════════════════════════════════════════════════════════");

    // Baseline: Expected performance
    let baseline_mean = Duration::from_micros(50); // 50 microseconds expected
    let tolerance = 1.2; // Allow 20% slower

    let stats = benchmark("Regression Check: Profile Generation", 1_000, || {
        let _profile = chrome_133();
        Ok(())
    });

    let regression_ratio = stats.mean.as_secs_f64() / baseline_mean.as_secs_f64();

    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Regression Analysis                                        │");
    println!("├────────────────────────────────────────────────────────────┤");
    println!(
        "│ Baseline:    {:.3} μs                                  │",
        baseline_mean.as_secs_f64() * 1_000_000.0
    );
    println!(
        "│ Current:     {:.3} μs                                  │",
        stats.mean.as_secs_f64() * 1_000_000.0
    );
    println!(
        "│ Ratio:       {:.2}x                                        │",
        regression_ratio
    );
    println!(
        "│ Tolerance:   {:.2}x                                        │",
        tolerance
    );

    if regression_ratio > tolerance {
        println!("│ Status:      ⚠️  REGRESSION DETECTED                      │");
    } else if regression_ratio < 0.9 {
        println!("│ Status:      ✅ PERFORMANCE IMPROVEMENT                   │");
    } else {
        println!("│ Status:      ✅ WITHIN EXPECTED RANGE                     │");
    }
    println!("└────────────────────────────────────────────────────────────┘");

    Ok(())
}

/// Main benchmark suite
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║        FINGERPRINT-RUST ADVANCED PERFORMANCE BENCHMARKS        ║");
    println!("║                                                                ║");
    println!("║  World-class performance testing and optimization techniques   ║");
    println!("║  Based on industry best practices and academic research        ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    bench_fingerprint_generation()?;
    bench_http_client_config()?;
    bench_comparative_analysis()?;
    bench_memory_profile()?;
    bench_regression_detection()?;

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                    BENCHMARKS COMPLETE");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("💡 Tips for further optimization:");
    println!("   1. Profile with 'cargo flamegraph' to find hot spots");
    println!("   2. Use 'cargo bloat' to identify binary size issues");
    println!("   3. Run 'cargo bench' for Criterion.rs integration");
    println!("   4. Enable LTO and opt-level=3 in release builds");
    println!("   5. Consider using jemalloc for better allocation performance");

    Ok(())
}
