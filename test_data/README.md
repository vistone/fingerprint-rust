# Test Data Directory

This directory contains test data for validating the fingerprinting framework.

## Directory Structure

```
test_data/
├── pcap/              # Real browser traffic captures
│   ├── chrome_136.pcap
│   ├── firefox_135.pcap
│   └── safari_17.pcap
├── expected/          # Expected test results
│   ├── chrome_136.json
│   ├── firefox_135.json
│   └── safari_17.json
└── synthetic/         # Synthetically generated test packets
    └── *.pcap
```

## Capturing Real Browser Traffic

### Prerequisites

```bash
# Install tcpdump (if not already installed)
sudo apt-get install tcpdump  # Debian/Ubuntu
sudo yum install tcpdump      # RHEL/CentOS
brew install tcpdump          # macOS
```

### Using the Capture Script

```bash
# Interactive mode
sudo ../scripts/capture_browser_traffic.sh

# Automated capture (all browsers)
sudo ../scripts/capture_browser_traffic.sh <<< "6"
```

### Manual Capture

```bash
# Capture Chrome traffic (30 seconds, port 443)
sudo tcpdump -i any -w test_data/pcap/chrome_136.pcap 'tcp port 443' &
TCPDUMP_PID=$!

# Open Chrome and visit HTTPS sites (google.com, github.com, etc.)
sleep 30

# Stop capture
sudo kill $TCPDUMP_PID
```

## Expected Results Format

Each PCAP file should have a corresponding JSON file in `expected/` directory:

```json
{
  "browser": "Chrome",
  "version": "136.0.6778.86",
  "os": "Linux",
  "confidence": 0.95,
  "layers": {
    "tcp": {
      "detected": true,
      "confidence": 0.75,
      "mss": 1460,
      "window_size": 65535,
      "ttl": 64
    },
    "tls": {
      "detected": true,
      "confidence": 0.90,
      "ja3": "...",
      "ja4": "t13d..."
    },
    "http2": {
      "detected": true,
      "confidence": 0.95,
      "hpack_signature": "..."
    }
  }
}
```

## Synthetic Test Data

For unit testing, synthetic PCAP files can be generated:

```rust
use fingerprint_core::packet_capture::*;

// Generate synthetic Chrome SYN packet
let pcap_header = PcapGlobalHeader::default();
let packet = create_chrome_syn_packet();

// Write to file
write_pcap("test_data/synthetic/chrome_syn.pcap", &[packet]);
```

## Usage in Tests

```rust
#[test]
fn test_real_chrome_136() {
    let result = analyze_pcap("test_data/pcap/chrome_136.pcap");
    let expected: ExpectedResult = 
        serde_json::from_str(include_str!("../test_data/expected/chrome_136.json")).unwrap();
    
    assert_eq!(result.browser, expected.browser);
    assert_eq!(result.version_major(), expected.version_major());
    assert!(result.confidence >= expected.confidence);
}
```

## Privacy & Security

⚠️ **Important**: Never commit PCAP files containing:
- Personal browsing data
- Authentication tokens
- Cookies or session IDs
- Private information

All sample PCAP files should be generated from:
- Public websites (google.com, example.com)
- Test environments
- Synthetic packet generators

The `.gitignore` is configured to exclude `*.pcap` files by default.

## Data Retention

- **Real PCAP files**: Delete after test validation (typically 24-48 hours)
- **Synthetic files**: Safe to keep indefinitely
- **Expected results**: Version controlled and safe to commit
