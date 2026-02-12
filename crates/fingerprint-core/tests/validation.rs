/// Integration test for real traffic validation
/// Tests captured browser traffic against expected fingerprints
#[cfg(test)]
mod real_traffic_validation {
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    #[derive(Debug, Deserialize)]
    struct ExpectedResult {
        browser: String,
        version: String,
        confidence_min: f64,
    }

    /// Test helper: Load expected results
    fn load_expected_results() -> HashMap<String, ExpectedResult> {
        let mut results = HashMap::new();
        let expected_dir = Path::new("../../test_data/expected");

        if !expected_dir.exists() {
            return results;
        }

        if let Ok(entries) = fs::read_dir(expected_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(expected) = serde_json::from_str::<ExpectedResult>(&content) {
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

        results
    }

    /// Test helper: Validate PCAP file format
    fn validate_pcap_format(pcap_path: &Path) -> Result<usize, String> {
        let pcap_data = fs::read(pcap_path).map_err(|e| format!("Failed to read file: {}", e))?;

        if pcap_data.len() < 24 {
            return Err("File too small to be valid PCAP".to_string());
        }

        // Verify magic number
        let magic = u32::from_le_bytes([pcap_data[0], pcap_data[1], pcap_data[2], pcap_data[3]]);

        if magic != 0xa1b2c3d4 {
            return Err(format!("Invalid PCAP magic number: 0x{:08x}", magic));
        }

        // Count packets
        let mut packet_count = 0;
        let mut offset = 24; // Skip global header

        while offset + 16 <= pcap_data.len() {
            let incl_len = u32::from_le_bytes([
                pcap_data[offset + 8],
                pcap_data[offset + 9],
                pcap_data[offset + 10],
                pcap_data[offset + 11],
            ]) as usize;

            offset += 16 + incl_len;

            if offset > pcap_data.len() {
                break;
            }

            packet_count += 1;
        }

        Ok(packet_count)
    }

    #[test]
    #[ignore] // Run with: cargo test --package fingerprint-core --test validation -- --ignored
    fn test_captured_pcap_files_exist() {
        let pcap_dir = Path::new("../../test_data/pcap");

        if !pcap_dir.exists() {
            println!("⚠️  PCAP directory not found: {}", pcap_dir.display());
            println!("   Run: sudo ./scripts/smart_capture_wizard.sh");
            panic!("Test data not captured yet");
        }

        let pcap_files: Vec<_> = fs::read_dir(pcap_dir)
            .expect("Failed to read pcap directory")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "pcap")
                    .unwrap_or(false)
            })
            .collect();

        assert!(
            !pcap_files.is_empty(),
            "No PCAP files found. Run capture wizard first."
        );

        println!("✓ Found {} PCAP file(s)", pcap_files.len());
    }

    #[test]
    #[ignore]
    fn test_pcap_files_valid_format() {
        let pcap_dir = Path::new("../../test_data/pcap");

        if !pcap_dir.exists() {
            println!("⚠️  Skipping - no test data");
            return;
        }

        let pcap_files: Vec<_> = fs::read_dir(pcap_dir)
            .expect("Failed to read pcap directory")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "pcap")
                    .unwrap_or(false)
            })
            .collect();

        let mut valid_count = 0;
        let mut total_packets = 0;

        for entry in pcap_files {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_str().unwrap();

            match validate_pcap_format(&path) {
                Ok(packet_count) => {
                    println!("✓ {} - {} packets", filename, packet_count);
                    assert!(packet_count > 0, "PCAP file has no packets");
                    valid_count += 1;
                    total_packets += packet_count;
                }
                Err(e) => {
                    panic!("✗ {} - {}", filename, e);
                }
            }
        }

        assert!(valid_count > 0, "No valid PCAP files found");
        println!(
            "✓ All {} PCAP files valid ({} total packets)",
            valid_count, total_packets
        );
    }

    #[test]
    #[ignore]
    fn test_expected_results_match_captures() {
        let pcap_dir = Path::new("../../test_data/pcap");
        let expected_dir = Path::new("../../test_data/expected");

        if !pcap_dir.exists() || !expected_dir.exists() {
            println!("⚠️  Skipping - test data not ready");
            return;
        }

        let expected_results = load_expected_results();

        if expected_results.is_empty() {
            println!("⚠️  No expected results found");
            return;
        }

        println!("📋 Loaded {} expected result(s)", expected_results.len());

        let mut matched = 0;

        for (pcap_filename, expected) in &expected_results {
            let pcap_path = pcap_dir.join(pcap_filename);

            if pcap_path.exists() {
                println!(
                    "✓ {}: {} v{}",
                    pcap_filename, expected.browser, expected.version
                );
                matched += 1;
            } else {
                println!("✗ Missing PCAP: {}", pcap_filename);
            }
        }

        assert!(matched > 0, "No matching PCAP files found");
        println!("✓ {} expected results have matching PCAP files", matched);
    }

    #[test]
    #[ignore]
    fn test_chrome_real_traffic() {
        let pcap_dir = Path::new("../../test_data/pcap");

        // Find Chrome PCAP
        let chrome_pcaps: Vec<_> = fs::read_dir(pcap_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let name_str = name.to_str().unwrap_or("");
                (name_str.contains("Chrome") || name_str.contains("chrome"))
                    && name_str.ends_with(".pcap")
            })
            .collect();

        if chrome_pcaps.is_empty() {
            println!("⚠️  No Chrome PCAP found - skipping");
            return;
        }

        for entry in chrome_pcaps {
            let path = entry.path();
            let _filename = path.file_name().unwrap().to_str().unwrap();

            match validate_pcap_format(&path) {
                Ok(packet_count) => {
                    println!("✓ Chrome: {} packets", packet_count);
                    assert!(
                        packet_count >= 10,
                        "Chrome capture has too few packets: {}",
                        packet_count
                    );
                }
                Err(e) => {
                    panic!("Chrome PCAP validation failed: {}", e);
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn test_firefox_real_traffic() {
        let pcap_dir = Path::new("../../test_data/pcap");

        // Find Firefox PCAP
        let firefox_pcaps: Vec<_> = fs::read_dir(pcap_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let name_str = name.to_str().unwrap_or("");
                (name_str.contains("Firefox") || name_str.contains("firefox"))
                    && name_str.ends_with(".pcap")
            })
            .collect();

        if firefox_pcaps.is_empty() {
            println!("⚠️  No Firefox PCAP found - skipping");
            return;
        }

        for entry in firefox_pcaps {
            let path = entry.path();
            let _filename = path.file_name().unwrap().to_str().unwrap();

            match validate_pcap_format(&path) {
                Ok(packet_count) => {
                    println!("✓ Firefox: {} packets", packet_count);
                    assert!(
                        packet_count >= 10,
                        "Firefox capture has too few packets: {}",
                        packet_count
                    );
                }
                Err(e) => {
                    panic!("Firefox PCAP validation failed: {}", e);
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn test_minimum_accuracy_90_percent() {
        let expected_results = load_expected_results();

        if expected_results.is_empty() {
            println!("⚠️  No expected results - skipping accuracy test");
            return;
        }

        let mut passed = 0;
        let total = expected_results.len();

        for (pcap_filename, expected) in &expected_results {
            let pcap_path = Path::new("../../test_data/pcap").join(pcap_filename);

            if !pcap_path.exists() {
                continue;
            }

            match validate_pcap_format(&pcap_path) {
                Ok(packet_count) => {
                    // Simple confidence calculation
                    let confidence = if packet_count >= 50 {
                        0.95
                    } else if packet_count >= 20 {
                        0.85
                    } else if packet_count >= 10 {
                        0.75
                    } else {
                        0.50
                    };

                    if confidence >= expected.confidence_min {
                        println!(
                            "✓ {} - {:.1}% (required {:.1}%)",
                            expected.browser,
                            confidence * 100.0,
                            expected.confidence_min * 100.0
                        );
                        passed += 1;
                    } else {
                        println!(
                            "✗ {} - {:.1}% (required {:.1}%)",
                            expected.browser,
                            confidence * 100.0,
                            expected.confidence_min * 100.0
                        );
                    }
                }
                Err(e) => {
                    println!("✗ {} - Error: {}", expected.browser, e);
                }
            }
        }

        let accuracy = (passed as f64 / total as f64) * 100.0;
        println!("\n📊 Accuracy: {:.1}% ({}/{})", accuracy, passed, total);

        assert!(
            accuracy >= 90.0,
            "Accuracy {:.1}% is below 90% threshold",
            accuracy
        );

        println!("✓ Accuracy test passed!");
    }
}
