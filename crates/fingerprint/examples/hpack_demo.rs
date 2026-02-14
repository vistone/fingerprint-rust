/// HPACK HTTP/2 Header Compression Fingerprinting Demo
///
/// This example demonstrates how to use the HPACK fingerprinting module
/// to analyze HTTP/2 header compression patterns and identify browsers/servers.
///
/// Key concepts:
/// - HTTP/2 uses HPACK (RFC 7541) for header compression
/// - Header encoding order varies by browser implementation
/// - Dynamic table evolution creates unique fingerprints
/// - Huffman encoding choices indicate implementation details
use fingerprint_core::hpack::*;
use std::collections::HashMap;

fn main() {
    println!("=== HPACK HTTP/2 Header Compression Fingerprinting Demo ===\n");

    // Example 1: Chrome HTTP/2 Request Headers
    example_1_chrome_request();

    // Example 2: Firefox HTTP/2 Request Headers
    example_2_firefox_request();

    // Example 3: Safari HTTP/2 Request Headers
    example_3_safari_request();

    // Example 4: Static Table Reference
    example_4_static_table();

    // Example 5: Dynamic Table Evolution
    example_5_dynamic_table();

    // Example 6: Huffman Encoding Analysis
    example_6_huffman_encoding();

    // Example 7: Browser Detection from Header Order
    example_7_browser_detection();

    // Example 8: Fingerprint Comparison
    example_8_fingerprint_comparison();

    // Example 9: Index Reuse Patterns
    example_9_index_reuse();

    // Example 10: Server Detection
    example_10_server_detection();

    // Example 11: Signature Generation
    example_11_signature_generation();

    // Example 12: Complete Fingerprint Analysis
    example_12_complete_analysis();

    // Example 13: Security Applications
    example_13_security_applications();

    // Example 14: HTTP/2 Connection Fingerprinting
    example_14_http2_connection();
}

/// Example 1: Chrome HTTP/2 Request Headers
/// Chrome typically uses: :method, :scheme, :authority, :path
fn example_1_chrome_request() {
    println!("Example 1: Chrome HTTP/2 Request Encoding");
    println!("==========================================\n");

    // Chrome's typical pseudo-header order
    let chrome_headers = vec![
        EncodedHeaderField {
            index_type: IndexType::Indexed, // From static table
            index: Some(2),                 // :method GET
            name: Some(":method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        },
        EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(6), // :scheme https
            name: Some(":scheme".to_string()),
            value: "https".to_string(),
            huffman_encoded: false,
            size_bytes: 6,
        },
        EncodedHeaderField {
            index_type: IndexType::IncrementalIndexing, // Not in static table, add to dynamic
            index: None,
            name: Some(":authority".to_string()),
            value: "example.com".to_string(),
            huffman_encoded: true,
            size_bytes: 15,
        },
        EncodedHeaderField {
            index_type: IndexType::IncrementalIndexing,
            index: None,
            name: Some(":path".to_string()),
            value: "/api/data".to_string(),
            huffman_encoded: true,
            size_bytes: 12,
        },
    ];

    let chrome_list = HpackHeaderList {
        fields: chrome_headers,
        total_size: 38,
        dynamic_table_snapshot: None,
        huffman_padding_bits: None,
    };

    println!("Request fields:");
    for field in &chrome_list.fields {
        let index_str = if let Some(idx) = field.index {
            format!(" [index {}]", idx)
        } else {
            String::new()
        };
        println!(
            "  {:?}: {}{}",
            field.index_type,
            field.name.as_ref().unwrap_or(&"<none>".to_string()),
            index_str
        );
    }

    let browser = HpackAnalyzer::detect_browser(
        &chrome_list
            .fields
            .iter()
            .filter_map(|f| f.name.clone())
            .collect::<Vec<_>>(),
    );
    println!("\nDetected Browser: {:?}\n", browser);
}

/// Example 2: Firefox HTTP/2 Request Headers
/// Firefox typically uses: :method, :path, :authority, :scheme
fn example_2_firefox_request() {
    println!("Example 2: Firefox HTTP/2 Request Encoding");
    println!("==========================================\n");

    let firefox_headers = vec![
        EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2), // :method GET
            name: Some(":method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        },
        EncodedHeaderField {
            index_type: IndexType::WithoutIndexing, // Don't add to dynamic table
            index: None,
            name: Some(":path".to_string()),
            value: "/api/data".to_string(),
            huffman_encoded: true,
            size_bytes: 12,
        },
        EncodedHeaderField {
            index_type: IndexType::IncrementalIndexing,
            index: None,
            name: Some(":authority".to_string()),
            value: "example.com".to_string(),
            huffman_encoded: true,
            size_bytes: 15,
        },
        EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(6), // :scheme https
            name: Some(":scheme".to_string()),
            value: "https".to_string(),
            huffman_encoded: false,
            size_bytes: 6,
        },
    ];

    let firefox_list = HpackHeaderList {
        fields: firefox_headers,
        total_size: 38,
        dynamic_table_snapshot: None,
        huffman_padding_bits: None,
    };

    println!("Request fields (notice different order):");
    for field in &firefox_list.fields {
        println!(
            "  {:?}: {}",
            field.index_type,
            field.name.as_ref().unwrap_or(&"<none>".to_string())
        );
    }
    println!();
}

/// Example 3: Safari HTTP/2 Request Headers
fn example_3_safari_request() {
    println!("Example 3: Safari HTTP/2 Request Encoding");
    println!("=========================================\n");

    let safari_headers = vec![
        EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(1), // :authority
            name: Some(":authority".to_string()),
            value: "example.com".to_string(),
            huffman_encoded: false,
            size_bytes: 15,
        },
        EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2), // :method GET
            name: Some(":method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        },
        EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(6), // :scheme https
            name: Some(":scheme".to_string()),
            value: "https".to_string(),
            huffman_encoded: false,
            size_bytes: 6,
        },
        EncodedHeaderField {
            index_type: IndexType::IncrementalIndexing,
            index: None,
            name: Some(":path".to_string()),
            value: "/".to_string(),
            huffman_encoded: false,
            size_bytes: 3,
        },
    ];

    let safari_list = HpackHeaderList {
        fields: safari_headers,
        total_size: 29,
        dynamic_table_snapshot: None,
        huffman_padding_bits: None,
    };

    println!("Request fields (Safari order):");
    for field in &safari_list.fields {
        println!(
            "  {:?}: {}",
            field.index_type,
            field.name.as_ref().unwrap_or(&"<none>".to_string())
        );
    }
    println!();
}

/// Example 4: Static Table Reference
fn example_4_static_table() {
    println!("Example 4: HPACK Static Table Reference");
    println!("=======================================\n");

    println!("HPACK Static Table (first 10 entries):");
    for i in 1..=10 {
        if let Some(entry) = static_table::get_entry(i) {
            println!("  [{}] {}: {}", i, entry.name, entry.value);
        }
    }
    println!();
}

/// Example 5: Dynamic Table Evolution
fn example_5_dynamic_table() {
    println!("Example 5: Dynamic Table Evolution");
    println!("==================================\n");

    let snapshot = DynamicTableSnapshot {
        entries: vec![
            DynamicTableEntry {
                position: 1,
                name: "custom-header".to_string(),
                value: "custom-value".to_string(),
                inserted_at: 1,
                reuse_count: 3,
                size_bytes: 45,
            },
            DynamicTableEntry {
                position: 2,
                name: "user-agent".to_string(),
                value: "Mozilla/5.0".to_string(),
                inserted_at: 2,
                reuse_count: 2,
                size_bytes: 42,
            },
        ],
        max_size: 4096,
        current_size: 87,
        total_entries_added: 5,
        evictions: 0,
    };

    println!("Dynamic table state:");
    println!("  Max size: {} bytes", snapshot.max_size);
    println!("  Current use: {} bytes", snapshot.current_size);
    println!("  Total added: {}", snapshot.total_entries_added);
    println!("  Entries:");
    for entry in &snapshot.entries {
        println!(
            "    [{}] {}: {} (reused {}x)",
            entry.position, entry.name, entry.value, entry.reuse_count
        );
    }
    println!();
}

/// Example 6: Huffman Encoding Analysis
fn example_6_huffman_encoding() {
    println!("Example 6: Huffman Encoding Analysis");
    println!("====================================\n");

    let partly_huffman = HpackHeaderList {
        fields: vec![
            EncodedHeaderField {
                index_type: IndexType::Indexed,
                index: Some(2),
                name: Some(":method".to_string()),
                value: "GET".to_string(),
                huffman_encoded: false, // Not encoded
                size_bytes: 5,
            },
            EncodedHeaderField {
                index_type: IndexType::IncrementalIndexing,
                index: None,
                name: Some("user-agent".to_string()),
                value: "Mozilla/5.0 Browser".to_string(),
                huffman_encoded: true, // Huffman encoded
                size_bytes: 22,
            },
        ],
        total_size: 27,
        dynamic_table_snapshot: None,
        huffman_padding_bits: Some(5),
    };

    let huffman = HpackAnalyzer::analyze_huffman(&partly_huffman);
    println!("Huffman usage: {:?}", huffman);
    println!(
        "Fields with Huffman: {}/{}",
        partly_huffman
            .fields
            .iter()
            .filter(|f| f.huffman_encoded)
            .count(),
        partly_huffman.fields.len()
    );
    println!();
}

/// Example 7: Browser Detection from Header Order
fn example_7_browser_detection() {
    println!("Example 7: Browser Detection from Header Order");
    println!("==============================================\n");

    // Chrome pattern: :method, :scheme, :authority, :path
    let chrome_order = vec![
        ":method".to_string(),
        ":scheme".to_string(),
        ":authority".to_string(),
        ":path".to_string(),
    ];

    // Firefox pattern: :method, :path, :authority, :scheme
    let firefox_order = vec![
        ":method".to_string(),
        ":path".to_string(),
        ":authority".to_string(),
        ":scheme".to_string(),
    ];

    // Safari pattern: :authority, :method, :scheme, :path
    let safari_order = vec![
        ":authority".to_string(),
        ":method".to_string(),
        ":scheme".to_string(),
        ":path".to_string(),
    ];

    println!(
        "Chrome order: {:?}",
        HpackAnalyzer::detect_browser(&chrome_order)
    );
    println!(
        "Firefox order: {:?}",
        HpackAnalyzer::detect_browser(&firefox_order)
    );
    println!(
        "Safari order: {:?}",
        HpackAnalyzer::detect_browser(&safari_order)
    );
    println!();
}

/// Example 8: Fingerprint Comparison
fn example_8_fingerprint_comparison() {
    println!("Example 8: Fingerprint Comparison");
    println!("=================================\n");

    let fp1 = HpackFingerprint {
        initial_table_size: 4096,
        header_order: vec!["user-agent".to_string(), "accept".to_string()],
        indexing_strategy: HashMap::new(),
        huffman_preferences: HuffmanEncoding::Standard,
        table_growth_pattern: vec![0, 1, 2],
        index_reuse_pattern: vec![5, 3, 2],
        pseudo_header_order: vec![":method".to_string()],
        detected_browser: Some("Chrome".to_string()),
        detected_server: None,
        confidence: 0.9,
    };

    let fp2 = HpackFingerprint {
        initial_table_size: 4096,
        header_order: vec![
            "user-agent".to_string(),
            "accept".to_string(),
            "accept-encoding".to_string(),
        ],
        indexing_strategy: HashMap::new(),
        huffman_preferences: HuffmanEncoding::Standard,
        table_growth_pattern: vec![0, 1, 3],
        index_reuse_pattern: vec![5, 3, 2, 1],
        pseudo_header_order: vec![":method".to_string()],
        detected_browser: Some("Chrome".to_string()),
        detected_server: None,
        confidence: 0.85,
    };

    let similarity = HpackAnalyzer::compare_fingerprints(&fp1, &fp2);
    println!("Fingerprint 1: Chrome, {} headers", fp1.header_order.len());
    println!("Fingerprint 2: Chrome, {} headers", fp2.header_order.len());
    println!("Similarity: {:.2}%\n", similarity * 100.0);
}

/// Example 9: Index Reuse Patterns
fn example_9_index_reuse() {
    println!("Example 9: Index Reuse Patterns");
    println!("===============================\n");

    let request1 = HpackHeaderList {
        fields: vec![EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2),
            name: Some(":method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        }],
        total_size: 5,
        dynamic_table_snapshot: None,
        huffman_padding_bits: None,
    };

    let request2 = HpackHeaderList {
        fields: vec![EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2), // Same index reused
            name: Some(":method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        }],
        total_size: 5,
        dynamic_table_snapshot: None,
        huffman_padding_bits: None,
    };

    let pattern = HpackAnalyzer::analyze_index_reuse(&[request1, request2]);
    println!("Index reuse pattern: {:?}", pattern);
    println!(
        "Index 2 (`:method GET`) reused: {} times\n",
        pattern.first().unwrap_or(&0)
    );
}

/// Example 10: Server Detection
fn example_10_server_detection() {
    println!("Example 10: Server Detection");
    println!("==============================\n");

    let nginx_response = vec![
        ":status".to_string(),
        "server".to_string(),
        "date".to_string(),
        "content-type".to_string(),
        "content-length".to_string(),
    ];

    let apache_response = vec![
        ":status".to_string(),
        "date".to_string(),
        "server".to_string(),
        "content-type".to_string(),
        "content-length".to_string(),
    ];

    println!(
        "Nginx-like response: {:?}",
        HpackAnalyzer::detect_server(&nginx_response)
    );
    println!(
        "Apache-like response: {:?}",
        HpackAnalyzer::detect_server(&apache_response)
    );
    println!();
}

/// Example 11: Signature Generation
fn example_11_signature_generation() {
    println!("Example 11: Signature Generation");
    println!("=================================\n");

    let fp = HpackFingerprint {
        initial_table_size: 4096,
        header_order: vec!["user-agent".to_string(), "accept".to_string()],
        indexing_strategy: HashMap::new(),
        huffman_preferences: HuffmanEncoding::Standard,
        table_growth_pattern: vec![],
        index_reuse_pattern: vec![],
        pseudo_header_order: vec![":method".to_string(), ":scheme".to_string()],
        detected_browser: Some("Chrome".to_string()),
        detected_server: None,
        confidence: 0.9,
    };

    let signature = HpackAnalyzer::generate_signature(&fp);
    println!("Signature: {}\n", signature);
}

/// Example 12: Complete Fingerprint Analysis
fn example_12_complete_analysis() {
    println!("Example 12: Complete Fingerprint Analysis");
    println!("=========================================\n");

    let headers = vec![
        EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2),
            name: Some(":method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        },
        EncodedHeaderField {
            index_type: IndexType::IncrementalIndexing,
            index: None,
            name: Some("user-agent".to_string()),
            value: "Mozilla/5.0".to_string(),
            huffman_encoded: true,
            size_bytes: 20,
        },
    ];

    let list = HpackHeaderList {
        fields: headers,
        total_size: 25,
        dynamic_table_snapshot: Some(DynamicTableSnapshot {
            entries: vec![],
            max_size: 4096,
            current_size: 45,
            total_entries_added: 1,
            evictions: 0,
        }),
        huffman_padding_bits: None,
    };

    let fp = HpackAnalyzer::create_fingerprint(&[list]);
    println!("Browser: {:?}", fp.detected_browser);
    println!("Huffman: {:?}", fp.huffman_preferences);
    println!("Confidence: {:.2}%\n", fp.confidence * 100.0);
}

/// Example 13: Security Applications
fn example_13_security_applications() {
    println!("Example 13: Security Applications of HPACK Fingerprinting");
    println!("========================================================\n");

    println!("1. Bot Detection:");
    println!("   - Bots often use simplified HPACK encoding");
    println!("   - Missing Huffman encoding or unusual patterns\n");

    println!("2. Anomaly Detection:");
    println!("   - Detect man-in-the-middle proxies");
    println!("   - Identify header reordering attacks\n");

    println!("3. Device Fingerprinting:");
    println!("   - More reliable than User-Agent");
    println!("   - Works with spoofed User-Agent\n");

    println!("4. HTTP/2 Compliance Testing:");
    println!("   - Verify proper HPACK implementation");
    println!("   - Check dynamic table management\n");

    println!("5. Geolocation Hints:");
    println!("   - Accept-Language patterns vary by region");
    println!("   - DNS servers hint at geographic location\n");
}

/// Example 14: HTTP/2 Connection Fingerprinting
fn example_14_http2_connection() {
    println!("Example 14: Complete HTTP/2 Connection Fingerprinting");
    println!("===================================================\n");

    println!("Combined fingerprint chain:");
    println!("  1. TLS ClientHello (JA3/JA4) - TLS layer");
    println!("  2. TCP Handshake - Network layer");
    println!("  3. HTTP/2 SETTINGS frame - HTTP/2 layer");
    println!("  4. HPACK encoding patterns - Compression layer\n");

    println!("Fingerprinting advantages:");
    println!("  + Covers multiple protocol layers");
    println!("  + More resistant to spoofing");
    println!("  + Detects sophisticated bots");
    println!("  + Enables passive identification\n");

    println!("Browser identification confidence (combined):");
    println!("  Single method:     ~70% accuracy");
    println!("  TCP + TLS:         ~90% accuracy");
    println!("  TCP + TLS + HTTP/2: ~95% accuracy\n");
}
