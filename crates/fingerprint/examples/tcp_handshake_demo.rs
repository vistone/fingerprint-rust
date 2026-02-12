/// TCP Handshake Fingerprinting Demo
///
/// Demonstrates TCP three-way handshake (SYN, SYN-ACK, ACK) analysis
/// for operating system and browser identification.
use fingerprint_core::tcp_handshake::*;

fn main() {
    println!("=== TCP Handshake Fingerprinting System ===\n");

    // Example 1: Basic TCP Option structures
    println!("1. TCP Option Creation:\n");

    let mss_option = TcpOption::mss(1460);
    println!("   MSS Option: {:?}", mss_option.option_type);

    let wscale_option = TcpOption::wscale(8);
    println!("   Window Scale: {:?}", wscale_option.option_type);

    let timestamp_option = TcpOption::timestamp(1234567890, 0);
    println!("   Timestamp: {:?}", timestamp_option.option_type);

    let sack_option = TcpOption::sack_permitted();
    println!("   SACK Permitted: {:?}\n", sack_option.option_type);

    // Example 2: Chrome on Windows 11 handshake
    println!("2. Chrome on Windows 11 Handshake Fingerprint:\n");

    let chrome_win_syn = SynCharacteristics {
        ip: IpCharacteristics {
            ttl: 128,
            dont_fragment: true,
            ip_id: 12345,
            ip_id_increment: None,
        },
        flags: TcpFlags {
            syn: true,
            ack: false,
            fin: false,
            rst: false,
            psh: false,
            urg: false,
        },
        window_size: 64240,
        options: vec![
            TcpOption::mss(1460),
            TcpOption::wscale(8),
            TcpOption::sack_permitted(),
            TcpOption::timestamp(1000000, 0),
        ],
        option_order: "MSS,WSCALE,SACK,Timestamp".to_string(),
    };

    let chrome_win_syn_ack = SynAckCharacteristics {
        ip: IpCharacteristics {
            ttl: 128,
            dont_fragment: true,
            ip_id: 54321,
            ip_id_increment: Some(1),
        },
        flags: TcpFlags {
            syn: true,
            ack: true,
            fin: false,
            rst: false,
            psh: false,
            urg: false,
        },
        window_size: 64240,
        options: vec![
            TcpOption::mss(1460),
            TcpOption::wscale(8),
            TcpOption::sack_permitted(),
            TcpOption::timestamp(2000000, 1000000),
        ],
        option_order: "MSS,WSCALE,SACK,Timestamp".to_string(),
    };

    let chrome_win_ack = AckCharacteristics {
        ip: IpCharacteristics {
            ttl: 128,
            dont_fragment: true,
            ip_id: 54322,
            ip_id_increment: Some(1),
        },
        flags: TcpFlags {
            syn: false,
            ack: true,
            fin: false,
            rst: false,
            psh: false,
            urg: false,
        },
        window_size: 64240,
        options: vec![TcpOption::timestamp(1000001, 2000000)],
        option_order: "Timestamp".to_string(),
    };

    let mut chrome_win_fingerprint =
        TcpHandshakeFingerprint::new(chrome_win_syn, chrome_win_syn_ack, chrome_win_ack);
    chrome_win_fingerprint.detected_os = Some("Windows 11".to_string());
    chrome_win_fingerprint.detected_browser = Some("Chrome 133".to_string());
    chrome_win_fingerprint.confidence = 0.95;

    println!(
        "   {}\n",
        TcpHandshakeAnalyzer::get_fingerprint_info(&chrome_win_fingerprint)
    );

    // Example 3: Firefox on Linux handshake
    println!("3. Firefox on Linux Handshake Fingerprint:\n");

    let firefox_linux_syn = SynCharacteristics {
        ip: IpCharacteristics {
            ttl: 64,
            dont_fragment: true,
            ip_id: 11111,
            ip_id_increment: None,
        },
        flags: TcpFlags {
            syn: true,
            ack: false,
            fin: false,
            rst: false,
            psh: false,
            urg: false,
        },
        window_size: 65535,
        options: vec![
            TcpOption::mss(1460),
            TcpOption::wscale(7),
            TcpOption::timestamp(3000000, 0),
            TcpOption::sack_permitted(),
        ],
        option_order: "MSS,WSCALE,Timestamp,SACK".to_string(),
    };

    let firefox_linux_syn_ack = SynAckCharacteristics {
        ip: IpCharacteristics {
            ttl: 64,
            dont_fragment: true,
            ip_id: 22222,
            ip_id_increment: Some(1),
        },
        flags: TcpFlags {
            syn: true,
            ack: true,
            fin: false,
            rst: false,
            psh: false,
            urg: false,
        },
        window_size: 65535,
        options: vec![
            TcpOption::mss(1460),
            TcpOption::wscale(7),
            TcpOption::timestamp(4000000, 3000000),
            TcpOption::sack_permitted(),
        ],
        option_order: "MSS,WSCALE,Timestamp,SACK".to_string(),
    };

    let firefox_linux_ack = AckCharacteristics {
        ip: IpCharacteristics {
            ttl: 64,
            dont_fragment: true,
            ip_id: 22223,
            ip_id_increment: Some(1),
        },
        flags: TcpFlags {
            syn: false,
            ack: true,
            fin: false,
            rst: false,
            psh: false,
            urg: false,
        },
        window_size: 65535,
        options: vec![TcpOption::timestamp(3000001, 4000000)],
        option_order: "Timestamp".to_string(),
    };

    let mut firefox_linux_fingerprint =
        TcpHandshakeFingerprint::new(firefox_linux_syn, firefox_linux_syn_ack, firefox_linux_ack);
    firefox_linux_fingerprint.detected_os = Some("Linux".to_string());
    firefox_linux_fingerprint.detected_browser = Some("Firefox 133".to_string());
    firefox_linux_fingerprint.confidence = 0.92;

    println!(
        "   {}\n",
        TcpHandshakeAnalyzer::get_fingerprint_info(&firefox_linux_fingerprint)
    );

    // Example 4: Operating System Detection
    println!("4. Operating System Detection from TTL:\n");

    let os_samples = vec![
        ((128, 128, 128), "Windows"),
        ((64, 64, 64), "Unix-like (Linux/macOS/iOS)"),
        ((127, 127, 127), "Windows (after 1 hop)"),
    ];

    for (ttl_seq, expected) in os_samples {
        if let Some(detected_os) = TcpHandshakeAnalyzer::detect_os(ttl_seq) {
            println!("   TTL {:?} → {} ✓", ttl_seq, detected_os);
        } else {
            println!("   TTL {:?} → Unknown (expected: {})", ttl_seq, expected);
        }
    }
    println!();

    // Example 5: Browser Detection from Option Order
    println!("5. Browser Detection from TCP Option Order:\n");

    let option_orders = vec![
        ("MSS,WSCALE,SACK,Timestamp", "Chrome/Chromium/Edge"),
        ("MSS,WSCALE,Timestamp,SACK", "Firefox"),
        ("MSS,WSCALE,Timestamp", "Safari"),
    ];

    for (order, expected) in option_orders {
        if let Some(detected_browser) = TcpHandshakeAnalyzer::detect_browser(order) {
            println!("   '{}' → {} ✓", order, detected_browser);
        } else {
            println!("   '{}' → Unknown (expected: {})", order, expected);
        }
    }
    println!();

    // Example 6: MSS Detection
    println!("6. Browser Detection from MSS (Maximum Segment Size):\n");

    let mss_samples = vec![(1460, "Chrome/Edge/Opera"), (1440, "Firefox")];

    for (mss, expected) in mss_samples {
        if let Some(detected) = TcpHandshakeAnalyzer::detect_from_mss(mss) {
            println!("   MSS {} → {} ✓", mss, detected);
        } else {
            println!("   MSS {} → Unknown (expected: {})", mss, expected);
        }
    }
    println!();

    // Example 7: Handshake Signature Comparison
    println!("7. TCP Handshake Signature Comparison:\n");

    println!("   Chrome Windows: {}", chrome_win_fingerprint.signature());
    println!(
        "   Firefox Linux:  {}\n",
        firefox_linux_fingerprint.signature()
    );

    // Example 8: Window Size Analysis
    println!("8. Window Size Evolution:\n");

    println!(
        "   Chrome Windows TTL: {:?}",
        chrome_win_fingerprint.ttl_sequence()
    );
    println!(
        "   Chrome Windows Window: {:?}",
        chrome_win_fingerprint.window_sequence()
    );
    println!(
        "   Firefox Linux TTL: {:?}",
        firefox_linux_fingerprint.ttl_sequence()
    );
    println!(
        "   Firefox Linux Window: {:?}\n",
        firefox_linux_fingerprint.window_sequence()
    );

    // Example 9: Pre-defined Signatures
    println!("9. Pre-defined Handshake Signatures:\n");

    println!("   Chrome on Windows 11:");
    println!("   {}", signatures::CHROME_WIN11);
    println!();
    println!("   Chrome on macOS:");
    println!("   {}", signatures::CHROME_MACOS);
    println!();
    println!("   Firefox on Windows:");
    println!("   {}", signatures::FIREFOX_WIN);
    println!();
    println!("   Safari on macOS:");
    println!("   {}\n", signatures::SAFARI_MACOS);

    // Example 10: Feature Summary
    println!("10. TCP Handshake Fingerprinting Features:\n");

    println!("   ✓ TCP Option Order Analysis");
    println!("   ✓ Window Size Tracking (SYN → SYN-ACK → ACK)");
    println!("   ✓ TTL Sequence Detection (OS identification)");
    println!("   ✓ MSS-based Browser Detection");
    println!("   ✓ IP Flag Analysis (DF, ID increment)");
    println!("   ✓ TCP Flag Verification");
    println!("   ✓ Pre-defined Signature Library");
    println!("   ✓ Confidence Scoring\n");

    // Example 11: Typical Fingerprint Chains
    println!("11. Typical TCP Handshake Signatures:\n");

    println!("   Chrome Windows 11:");
    println!("   └─ TTL: 128, MSS: 1460, Options: MSS,WSCALE,SACK,Timestamp");
    println!();
    println!("   Firefox on Linux:");
    println!("   └─ TTL: 64, MSS: 1460, Options: MSS,WSCALE,Timestamp,SACK");
    println!();
    println!("   Safari on macOS:");
    println!("   └─ TTL: 64, MSS: 1460, Options: MSS,WSCALE,Timestamp\n");

    // Example 12: Security Applications
    println!("12. Security Applications:\n");

    println!("   ✓ Anomaly Detection");
    println!("   └─ Detect non-standard TCP behavior");
    println!();
    println!("   ✓ Bot Detection");
    println!("   └─ Many bots have abnormal TCP fingerprints");
    println!();
    println!("   ✓ OS Fingerprinting");
    println!("   └─ Identify operating system from handshake");
    println!();
    println!("   ✓ Browser Identification");
    println!("   └─ Combined with TLS fingerprint for accuracy");
    println!();
    println!("   ✓ Geolocation Hints");
    println!("   └─ TCP window sizes hint at geographic location\n");

    // Example 13: Structure Summary
    println!("13. Structure Overview:\n");

    println!("   TcpOption");
    println!("   ├─ MSS (v2, 1460 bytes)");
    println!("   ├─ WSCALE (v3, shift=8)");
    println!("   ├─ SACK (v4)");
    println!("   └─ Timestamp (v8)\n");

    println!("   TcpFlags");
    println!("   ├─ SYN");
    println!("   ├─ ACK");
    println!("   └─ Other control flags\n");

    println!("   IpCharacteristics");
    println!("   ├─ TTL (Time To Live)");
    println!("   ├─ DF (Don't Fragment)");
    println!("   └─ IP ID patterns\n");

    println!("   TcpHandshakeFingerprint");
    println!("   ├─ SYN packet");
    println!("   ├─ SYN-ACK response");
    println!("   ├─ ACK confirmation");
    println!("   └─ Detection results\n");

    // Example 14: Integration with TLS
    println!("14. Combined TCP + TLS Fingerprinting:\n");

    println!("   Traditional approach:");
    println!("   └─ TLS fingerprint only (JA3, JA4)");
    println!();
    println!("   Enhanced approach:");
    println!("   └─ TCP handshake + TLS fingerprint");
    println!("      = Much higher accuracy");
    println!("      = Better bot detection");
    println!("      = Cross-platform identification\n");

    println!("=== Summary ===\n");
    println!("TCP handshake fingerprinting provides:");
    println!("✓ OS-level device identification");
    println!("✓ Browser/application detection");
    println!("✓ Anomaly detection capability");
    println!("✓ Complements TLS fingerprinting");
    println!("✓ Works even with encrypted connections\n");
}
