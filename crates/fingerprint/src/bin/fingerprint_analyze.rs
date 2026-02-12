/// PCAP Traffic Analyzer
/// Analyzes captured browser traffic and generates fingerprint reports
use fingerprint_core::ja3_database::{BrowserMatch, JA3Database};
use fingerprint_core::packet_capture::*;
use fingerprint_core::signature::ClientHelloSignature;
use fingerprint_core::tls_parser::find_client_hello;
use fingerprint_core::{find_settings_frame, Http2SettingsMatcher};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct BrowserFingerprint {
    window_size: Option<u16>,
    ttl: Option<u8>,
    packet_count: usize,
    confidence: f64,
    // HTTP/2 fields
    http2_settings: Option<HashMap<u16, u32>>,
    http2_browser: Option<String>,
    http2_confidence: Option<f64>,
    // TLS fields
    tls_signature: Option<ClientHelloSignature>,
    ja3_fingerprint: Option<String>,
    tls_confidence: Option<f64>,
    // JA3 database match
    ja3_match: Option<BrowserMatch>,
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
    let mut http2_settings: Option<HashMap<u16, u32>> = None;
    let mut http2_browser: Option<String> = None;
    let mut http2_confidence: Option<f64> = None;
    let matcher = Http2SettingsMatcher::new();

    // TLS tracking
    let mut tls_signature: Option<ClientHelloSignature> = None;
    let mut ja3_fingerprint: Option<String> = None;
    let mut tls_confidence: Option<f64> = None;
    let mut ja3_match: Option<BrowserMatch> = None;

    // Initialize JA3 database
    let ja3_db = JA3Database::new();

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

                if let Some((tcp, tcp_payload)) = PacketParser::parse_tcp(rest) {
                    window_sizes.push(tcp.window_size);
                    tcp_packets.push(tcp);
                    packet_count += 1;

                    // Try to find HTTP/2 SETTINGS frame (only if not found yet)
                    if http2_settings.is_none() && !tcp_payload.is_empty() {
                        if let Some(settings_frame) = find_settings_frame(tcp_payload) {
                            let settings = settings_frame.to_map();
                            let (browser, conf) = matcher.match_browser(&settings);
                            http2_settings = Some(settings);
                            http2_browser = Some(browser.to_string());
                            http2_confidence = Some(conf);
                        }
                    }

                    // Try to find TLS ClientHello (only if not found yet)
                    if tls_signature.is_none() && !tcp_payload.is_empty() {
                        if let Some(client_hello) = find_client_hello(tcp_payload) {
                            // Calculate JA3 fingerprint
                            let ja3 = fingerprint_core::ja3::JA3::from_client_hello(&client_hello);
                            let ja3_string = ja3.to_string();

                            // Match against JA3 database
                            let db_match = ja3_db.match_ja3(&ja3_string);

                            // Estimate confidence based on signature completeness
                            let mut tls_conf: f64 = 0.70; // Base confidence
                            if !client_hello.cipher_suites.is_empty() {
                                tls_conf += 0.10;
                            }
                            if !client_hello.extensions.is_empty() {
                                tls_conf += 0.10;
                            }
                            if client_hello.alpn.is_some() {
                                tls_conf += 0.05;
                            }
                            if client_hello.sni.is_some() {
                                tls_conf += 0.05;
                            }

                            tls_signature = Some(client_hello);
                            ja3_fingerprint = Some(ja3_string);
                            tls_confidence = Some(tls_conf.min(1.0));
                            ja3_match = db_match;
                        }
                    }
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
    let mut confidence = calculate_confidence(packet_count, &tcp_packets, avg_ttl);

    // Enhance confidence with HTTP/2 fingerprint if detected
    if let Some(http2_conf) = http2_confidence {
        if http2_conf >= 0.90 {
            confidence += 0.15; // High confidence HTTP/2 match
        } else if http2_conf >= 0.75 {
            confidence += 0.10; // Medium confidence HTTP/2 match
        } else if http2_conf >= 0.60 {
            confidence += 0.05; // Low confidence HTTP/2 match
        }
        confidence = confidence.min(1.0);
    }

    // Enhance confidence with TLS fingerprint if detected
    if let Some(tls_conf) = tls_confidence {
        if tls_conf >= 0.90 {
            confidence += 0.15; // High confidence TLS match
        } else if tls_conf >= 0.80 {
            confidence += 0.12; // Good confidence TLS match
        } else if tls_conf >= 0.70 {
            confidence += 0.10; // Medium confidence TLS match
        }
        confidence = confidence.min(1.0);
    }

    // Additional boost from JA3 database match
    if let Some(ref match_info) = ja3_match {
        let ja3_boost = match_info.confidence * 0.10; // Up to 10% boost
        confidence += ja3_boost;
        confidence = confidence.min(1.0);
    }

    Ok(BrowserFingerprint {
        window_size: avg_window_size,
        ttl: avg_ttl,
        packet_count,
        confidence,
        http2_settings,
        http2_browser,
        http2_confidence,
        tls_signature,
        ja3_fingerprint,
        tls_confidence,
        ja3_match,
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

    // TTL value reasonableness (enhanced logic for real-world scenarios)
    if let Some(ttl_val) = ttl {
        if ttl_val >= 32 && ttl_val <= 128 {
            // Normal TTL range - full score
            confidence += 0.25;
        } else if ttl_val >= 8 && ttl_val < 32 {
            // Low TTL (likely multi-hop network, VPN, proxy) - partial score
            // Still indicates real network traffic, just heavily routed
            confidence += 0.20;
        } else if ttl_val >= 1 && ttl_val < 8 {
            // Very low TTL (extreme multi-hop) - minimal score
            // Unusual but not impossible in real networks
            confidence += 0.10;
        } else if ttl_val > 128 {
            // High TTL (possibly Windows with TTL=128+) - partial score
            confidence += 0.15;
        }
        // TTL = 0 gets no score (invalid)
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

    // HTTP/2 SETTINGS (if detected)
    if let Some(settings) = &fp.http2_settings {
        println!("\n  HTTP/2 SETTINGS:");
        if let Some(&window_size) = settings.get(&4) {
            println!(
                "    Window Size: {} bytes ({} KB)",
                window_size,
                window_size / 1024
            );
        }
        if let Some(&max_conc) = settings.get(&3) {
            println!("    Max Streams: {}", max_conc);
        }
        if let Some(&enable_push) = settings.get(&2) {
            let push_status = if enable_push == 1 {
                "Enabled"
            } else {
                "Disabled"
            };
            println!("    Server Push: {}", push_status);
        }
        if let (Some(browser), Some(conf)) = (&fp.http2_browser, fp.http2_confidence) {
            println!(
                "    HTTP/2 Match: {} ({:.1}% confidence)",
                browser,
                conf * 100.0
            );
        }
    }

    // TLS ClientHello (if detected)
    if let Some(signature) = &fp.tls_signature {
        println!("\n  TLS ClientHello:");
        println!("    Version: {:?}", signature.version);
        println!("    Ciphers: {} suites", signature.cipher_suites.len());
        println!("    Extensions: {} detected", signature.extensions.len());

        if let Some(alpn) = &signature.alpn {
            println!("    ALPN: {}", alpn);
        }
        if let Some(sni) = &signature.sni {
            println!("    SNI: {}", sni);
        }
        if let Some(ja3) = &fp.ja3_fingerprint {
            let short_ja3 = if ja3.len() > 50 {
                format!("{}...", &ja3[..50])
            } else {
                ja3.clone()
            };
            println!("    JA3: {}", short_ja3);
        }
        if let Some(conf) = fp.tls_confidence {
            println!("    TLS Match: {:.1}% confidence", conf * 100.0);
        }

        // JA3 database match
        if let Some(ref match_info) = fp.ja3_match {
            println!("\n  Browser Identification:");
            println!(
                "    Detected: {} {}",
                match_info.browser, match_info.version
            );
            println!(
                "    Match Confidence: {:.1}%",
                match_info.confidence * 100.0
            );
            if let Some(ref notes) = match_info.notes {
                println!("    Notes: {}", notes);
            }
        }
    }

    println!("\n  Overall Confidence: {:.1}%", fp.confidence * 100.0);

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
