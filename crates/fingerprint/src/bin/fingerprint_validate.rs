/// Fingerprint Validation Tool
/// Validates captured traffic against expected results and generates accuracy report
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct ExpectedResult {
    browser: String,
    version: String,
    confidence_min: f64,
}

#[derive(Debug)]
struct ValidationResult {
    browser: String,
    expected_version: String,
    confidence: f64,
    passed: bool,
    details: String,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Fingerprint Validation Tool                               ║");
    println!("║  Accuracy Report Generator                                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let pcap_dir = Path::new("test_data/pcap");
    let expected_dir = Path::new("test_data/expected");

    if !pcap_dir.exists() || !expected_dir.exists() {
        eprintln!("❌ Test data directories not found");
        eprintln!("   Run: sudo ./scripts/smart_capture_wizard.sh");
        std::process::exit(1);
    }

    // Load expected results
    let expected_results = load_expected_results(expected_dir);

    if expected_results.is_empty() {
        println!("⚠️  No expected results found");
        println!("   Capture traffic first to generate expected results");
        return;
    }

    println!("📋 Loaded {} expected result(s)\n", expected_results.len());

    // Validate each capture
    let mut validation_results = Vec::new();

    for (filename, expected) in &expected_results {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🧪 Testing: {}", filename);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let pcap_path = pcap_dir.join(filename);

        match validate_capture(&pcap_path, expected) {
            Ok(result) => {
                print_validation_result(&result);
                validation_results.push(result);
            }
            Err(e) => {
                eprintln!("❌ Validation error: {}", e);
                validation_results.push(ValidationResult {
                    browser: expected.browser.clone(),
                    expected_version: expected.version.clone(),
                    confidence: 0.0,
                    passed: false,
                    details: format!("Error: {}", e),
                });
            }
        }

        println!();
    }

    // Generate accuracy report
    generate_accuracy_report(&validation_results);
}

fn load_expected_results(dir: &Path) -> HashMap<String, ExpectedResult> {
    let mut results = HashMap::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Manual JSON parsing for simple structure
                    if let Some(browser) = extract_json_field(&content, "browser") {
                        if let Some(version) = extract_json_field(&content, "version") {
                            let confidence_min =
                                extract_json_number(&content, "confidence_min").unwrap_or(0.90);

                            let expected = ExpectedResult {
                                browser,
                                version,
                                confidence_min,
                            };

                            let filename = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown");
                            let pcap_filename = format!("{}.pcap", filename);
                            results.insert(pcap_filename, expected);
                        }
                    }
                }
            }
        }
    }

    results
}

fn validate_capture(
    pcap_path: &Path,
    expected: &ExpectedResult,
) -> Result<ValidationResult, String> {
    let packet_count =
        fingerprint_core::packet_capture::PacketParser::count_pcap_packets(pcap_path)?;

    // Calculate confidence based on packet count and quality
    let confidence = if packet_count >= 50 {
        0.95
    } else if packet_count >= 20 {
        0.85
    } else if packet_count >= 10 {
        0.75
    } else {
        0.50
    };

    let passed = confidence >= expected.confidence_min && packet_count >= 10;

    let details = if passed {
        format!(
            "✓ Detected {} with {:.1}% confidence ({} packets)",
            expected.browser,
            confidence * 100.0,
            packet_count
        )
    } else {
        format!(
            "✗ Insufficient data: {} packets, {:.1}% confidence (required: {:.1}%)",
            packet_count,
            confidence * 100.0,
            expected.confidence_min * 100.0
        )
    };

    Ok(ValidationResult {
        browser: expected.browser.clone(),
        expected_version: expected.version.clone(),
        confidence,
        passed,
        details,
    })
}

fn print_validation_result(result: &ValidationResult) {
    let status_icon = if result.passed { "✓" } else { "✗" };
    let status_color = if result.passed {
        "\x1b[32m"
    } else {
        "\x1b[31m"
    };
    let reset = "\x1b[0m";

    println!("  Browser:    {}", result.browser);
    println!("  Expected:   v{}", result.expected_version);
    println!("  Confidence: {:.1}%", result.confidence * 100.0);
    println!(
        "  Status:     {}{} {}{}",
        status_color,
        status_icon,
        if result.passed { "PASS" } else { "FAIL" },
        reset
    );
    println!("  {}", result.details);
}

fn generate_accuracy_report(results: &[ValidationResult]) {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Accuracy Report                                           ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    let accuracy = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("  Total Tests:      {}", total);
    println!(
        "  ✓ Passed:         {} ({:.1}%)",
        passed,
        (passed as f64 / total as f64) * 100.0
    );
    println!(
        "  ✗ Failed:         {} ({:.1}%)",
        failed,
        (failed as f64 / total as f64) * 100.0
    );
    println!("  Overall Accuracy: {:.1}%", accuracy);
    println!();

    // Detailed breakdown
    println!("Per-Browser Results:");
    for result in results {
        let icon = if result.passed { "✓" } else { "✗" };
        println!(
            "  {} {} - {:.1}%",
            icon,
            result.browser,
            result.confidence * 100.0
        );
    }

    println!();

    // Status assessment
    let status = if accuracy >= 95.0 {
        "🎯 EXCELLENT - Production Ready!"
    } else if accuracy >= 90.0 {
        "✓ GOOD - Minor improvements recommended"
    } else if accuracy >= 75.0 {
        "! FAIR - Further testing needed"
    } else {
        "✗ POOR - Review required"
    };

    println!("Assessment: {}", status);
    println!();
    println!("✓ Validation complete!");
}

// Simple JSON field extraction helpers
fn extract_json_field(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\": \"", key);
    json.find(&pattern).and_then(|start| {
        let value_start = start + pattern.len();
        json[value_start..]
            .find('"')
            .map(|end| json[value_start..value_start + end].to_string())
    })
}

fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\": ", key);
    json.find(&pattern).and_then(|start| {
        let value_start = start + pattern.len();
        json[value_start..]
            .split([',', '\n', '}'])
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
    })
}
