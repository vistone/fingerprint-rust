/// PCAP Traffic Analyzer
/// Analyzes captured browser traffic and generates fingerprint reports
use fingerprint_core::packet_capture::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct BrowserFingerprint {
    window_size: Option<u16>,
    ttl: Option<u8>,
    packet_count: usize,
    confidence: f64,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  PCAP Traffic Analyzer                                     ║");
    println!("║  Real Browser Fingerprint Analysis                        ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let pcap_dir = Path::new("test_data/pcap");

    if !pcap_dir.exists() {
        eprintln!("❌ Directory not found: {}", pcap_dir.display());
        eprintln!("   Run: sudo ./scripts/smart_capture_wizard.sh");
        std::process::exit(1);
    }

    // Find all PCAP files
    let pcap_files: Vec<_> = fs::read_dir(pcap_dir)
        .expect("Failed to read pcap directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "pcap")
                .unwrap_or(false)
        })
        .collect();

    if pcap_files.is_empty() {
        println!("⚠️  No PCAP files found in {}", pcap_dir.display());
        println!("   Run capture first: sudo ./scripts/smart_capture_wizard.sh");
        return;
    }

    println!("📊 Found {} PCAP file(s)\n", pcap_files.len());

    let mut results = HashMap::new();

    for entry in pcap_files {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📁 Analyzing: {}", filename);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match analyze_pcap(&path) {
            Ok(fingerprint) => {
                print_fingerprint_report(filename, &fingerprint);
                results.insert(filename.to_string(), fingerprint);
            }
            Err(e) => {
                eprintln!("❌ Error analyzing {}: {}", filename, e);
            }
        }

        println!();
    }

    // Summary
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Analysis Summary                                          ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    for (filename, fingerprint) in &results {
        let browser_name = extract_browser_name(filename);
        let confidence_icon = if fingerprint.confidence >= 0.90 {
            "✓"
        } else if fingerprint.confidence >= 0.75 {
            "!"
        } else {
            "✗"
        };

        println!(
            "  {} {} - Confidence: {:.1}%",
            confidence_icon,
            browser_name,
            fingerprint.confidence * 100.0
        );
    }

    println!();
    println!("✓ Analysis complete!");
    println!("  Next: cargo run --bin fingerprint_validate");
}

fn analyze_pcap(path: &Path) -> Result<BrowserFingerprint, String> {
    // Read PCAP file
    let pcap_data = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    if pcap_data.len() < 24 {
        return Err("File too small to be valid PCAP".to_string());
    }

    // Verify magic number
    let magic = u32::from_le_bytes([pcap_data[0], pcap_data[1], pcap_data[2], pcap_data[3]]);
    if magic != 0xa1b2c3d4 {
        return Err(format!("Invalid PCAP magic number: 0x{:08x}", magic));
    }

    // Parse packets
    let mut offset = 24; // Skip global header
    let mut packet_count = 0;
    let mut tcp_packets = Vec::new();
    let mut window_sizes = Vec::new();
    let mut ttls = Vec::new();

    while offset + 16 <= pcap_data.len() {
        // Parse packet header
        let incl_len = u32::from_le_bytes([
            pcap_data[offset + 8],
            pcap_data[offset + 9],
            pcap_data[offset + 10],
            pcap_data[offset + 11],
        ]) as usize;

        offset += 16;

        if offset + incl_len > pcap_data.len() {
            break;
        }

        let packet_data = &pcap_data[offset..offset + incl_len];

        // Parse Ethernet → IPv4 → TCP
        if let Some((_, rest)) = PacketParser::parse_ethernet(packet_data) {
            if let Some((ipv4, rest)) = PacketParser::parse_ipv4(rest) {
                ttls.push(ipv4.ttl);

                if let Some((tcp, _)) = PacketParser::parse_tcp(rest) {
                    window_sizes.push(tcp.window_size);
                    tcp_packets.push(tcp);
                    packet_count += 1;
                }
            }
        }

        offset += incl_len;
    }

    if packet_count == 0 {
        return Err("No TCP packets found".to_string());
    }

    // Analyze TCP fingerprints
    let avg_window_size = if !window_sizes.is_empty() {
        let sum: u64 = window_sizes.iter().map(|&w| w as u64).sum();
        Some((sum / window_sizes.len() as u64) as u16)
    } else {
        None
    };

    let avg_ttl = if !ttls.is_empty() {
        let sum: u64 = ttls.iter().map(|&t| t as u64).sum();
        Some((sum / ttls.len() as u64) as u8)
    } else {
        None
    };

    // Calculate confidence based on data quality
    let confidence = calculate_confidence(packet_count, &tcp_packets, avg_ttl);

    Ok(BrowserFingerprint {
        window_size: avg_window_size,
        ttl: avg_ttl,
        packet_count,
        confidence,
    })
}

fn calculate_confidence(packet_count: usize, tcp_packets: &[TcpHeader], ttl: Option<u8>) -> f64 {
    let mut confidence: f64 = 0.0;

    // Packet count factor (more packets = higher confidence)
    if packet_count >= 50 {
        confidence += 0.40;
    } else if packet_count >= 20 {
        confidence += 0.30;
    } else if packet_count >= 10 {
        confidence += 0.20;
    }

    // TCP SYN packet presence
    let has_syn = tcp_packets.iter().any(|tcp| tcp.syn());
    if has_syn {
        confidence += 0.20;
    }

    // Window size consistency
    let window_sizes: Vec<u16> = tcp_packets.iter().map(|tcp| tcp.window_size).collect();
    if window_sizes.len() > 1 {
        let avg = window_sizes.iter().map(|&w| w as f64).sum::<f64>() / window_sizes.len() as f64;
        let variance = window_sizes
            .iter()
            .map(|&w| {
                let diff = w as f64 - avg;
                diff * diff
            })
            .sum::<f64>()
            / window_sizes.len() as f64;

        if variance < 10000.0 {
            confidence += 0.15;
        }
    }

    // TTL value reasonableness
    if let Some(ttl_val) = ttl {
        if ttl_val >= 32 && ttl_val <= 128 {
            confidence += 0.25;
        }
    }

    confidence.min(1.0)
}

fn print_fingerprint_report(filename: &str, fp: &BrowserFingerprint) {
    let browser_name = extract_browser_name(filename);

    println!("  Browser: {}", browser_name);
    println!("  Packets: {}", fp.packet_count);

    if let Some(window) = fp.window_size {
        println!("  Window Size: {}", window);
    }

    if let Some(ttl) = fp.ttl {
        println!("  TTL: {}", ttl);

        // Infer OS from TTL
        let os_guess = if ttl <= 64 {
            "Linux/Unix"
        } else if ttl <= 128 {
            "Windows"
        } else {
            "Unknown"
        };
        println!("  OS (guess): {}", os_guess);
    }

    println!("  Confidence: {:.1}%", fp.confidence * 100.0);

    let status = if fp.confidence >= 0.90 {
        "✓ EXCELLENT"
    } else if fp.confidence >= 0.75 {
        "! GOOD"
    } else if fp.confidence >= 0.50 {
        "⚠ FAIR"
    } else {
        "✗ POOR"
    };

    println!("  Status: {}", status);
}

fn extract_browser_name(filename: &str) -> String {
    if filename.contains("Chrome") || filename.contains("chrome") {
        "Chrome".to_string()
    } else if filename.contains("Firefox") || filename.contains("firefox") {
        "Firefox".to_string()
    } else if filename.contains("Safari") || filename.contains("safari") {
        "Safari".to_string()
    } else {
        filename.trim_end_matches(".pcap").to_string()
    }
}
