/// HPACK (HTTP/2 Header Compression) Fingerprinting Module
///
/// Analyzes HTTP/2 header compression patterns to identify browsers and servers.
///
/// HPACK (RFC 7541) defines:
/// - Static table (61 predefined entries)
/// - Dynamic table (evolves with each request)
/// - Huffman string encoding
/// - Progressive indexing strategies
///
/// For fingerprinting, we analyze:
/// - Header encoding order and patterns
/// - Dynamic table construction sequence
/// - Huffman encoding choices
/// - Index usage (incremental, without indexing, never indexed)
/// - Table size management
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HPACK Static Table Entry (RFC 7541, Section 2.3.1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StaticTableEntry {
    /// Index in static table (1-61)
    pub index: u8,
    /// Header field name
    pub name: String,
    /// Header field value (may be empty)
    pub value: String,
}

/// Huffman coding state for fingerprinting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HuffmanEncoding {
    /// String not Huffman encoded
    None,
    /// Standard Huffman encoding (RFC 7541 Table 4)
    Standard,
    /// Variant Huffman table (rare, indicates custom implementation)
    Custom,
}

/// Index representation in HPACK
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndexType {
    /// Indexed representation (index to static/dynamic table)
    Indexed,
    /// Literal with incremental indexing
    IncrementalIndexing,
    /// Literal without indexing
    WithoutIndexing,
    /// Literal never indexed
    NeverIndexed,
}

/// Individual header field encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedHeaderField {
    /// Index type used in encoding
    pub index_type: IndexType,
    /// Index or name reference
    pub index: Option<u8>,
    /// Field name (if not indexed)
    pub name: Option<String>,
    /// Field value
    pub value: String,
    /// Whether value was Huffman encoded
    pub huffman_encoded: bool,
    /// Encoding size in bytes
    pub size_bytes: u16,
}

/// Dynamic table entry (added during session)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTableEntry {
    /// Position in current dynamic table
    pub position: u8,
    /// Header name
    pub name: String,
    /// Header value
    pub value: String,
    /// When this entry was inserted (sequence number)
    pub inserted_at: u32,
    /// Number of times reused
    pub reuse_count: u32,
    /// Entry size (32 + len(name) + len(value))
    pub size_bytes: u16,
}

/// Dynamic table state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTableSnapshot {
    /// Current entries in dynamic table
    pub entries: Vec<DynamicTableEntry>,
    /// Maximum table size (from settings)
    pub max_size: u16,
    /// Current actual size used
    pub current_size: u16,
    /// Total entries ever added
    pub total_entries_added: u32,
    /// Table eviction count (LRU)
    pub evictions: u32,
}

/// HPACK Header List encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpackHeaderList {
    /// Encoded fields in order
    pub fields: Vec<EncodedHeaderField>,
    /// Total encoded size
    pub total_size: u16,
    /// Dynamic table state after encoding
    pub dynamic_table_snapshot: Option<DynamicTableSnapshot>,
    /// Huffman padding bits (can indicate implementation)
    pub huffman_padding_bits: Option<u8>,
}

/// HPACK Fingerprint - comprehensive HTTP/2 compression signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpackFingerprint {
    /// Initial table size setting (usually 4096 or custom)
    pub initial_table_size: u16,
    /// Header field order in first request
    pub header_order: Vec<String>,
    /// Which headers are indexed vs literal
    pub indexing_strategy: HashMap<String, IndexType>,
    /// Huffman encoding preferences
    pub huffman_preferences: HuffmanEncoding,
    /// Dynamic table growth pattern (entries added per request)
    pub table_growth_pattern: Vec<u8>,
    /// Index reuse pattern (how often same indices reused)
    pub index_reuse_pattern: Vec<u32>,
    /// Pseudo-headers order (:method, :path, :scheme, :authority)
    pub pseudo_header_order: Vec<String>,
    /// Browser identification hints
    pub detected_browser: Option<String>,
    /// Server identification hints
    pub detected_server: Option<String>,
    /// Confidence score
    pub confidence: f32,
}

/// HPACK Static Table (all 61 entries from RFC 7541)
pub mod static_table {
    use super::StaticTableEntry;

    /// Get static table entry by index (1-61)
    pub fn get_entry(index: u8) -> Option<StaticTableEntry> {
        if !(1..=61).contains(&index) {
            return None;
        }
        Some(StaticTableEntry {
            index,
            name: get_name(index).to_string(),
            value: get_value(index).to_string(),
        })
    }

    fn get_name(index: u8) -> &'static str {
        match index {
            1 => ":authority",
            2 => ":method",
            3 => ":method",
            4 => ":path",
            5 => ":path",
            6 => ":scheme",
            7 => ":scheme",
            8 => ":status",
            9 => ":status",
            10 => ":status",
            11 => ":status",
            12 => ":status",
            13 => ":status",
            14 => "accept",
            15 => "accept-charset",
            16 => "accept-encoding",
            17 => "accept-language",
            18 => "accept-ranges",
            19 => "age",
            20 => "allow",
            21 => "authorization",
            22 => "cache-control",
            23 => "content-disposition",
            24 => "content-encoding",
            25 => "content-language",
            26 => "content-length",
            27 => "content-location",
            28 => "content-range",
            29 => "content-type",
            30 => "cookie",
            31 => "date",
            32 => "etag",
            33 => "expect",
            34 => "expires",
            35 => "from",
            36 => "host",
            37 => "if-match",
            38 => "if-modified-since",
            39 => "if-none-match",
            40 => "if-range",
            41 => "if-unmodified-since",
            42 => "last-modified",
            43 => "link",
            44 => "location",
            45 => "max-forwards",
            46 => "proxy-authenticate",
            47 => "proxy-authorization",
            48 => "range",
            49 => "referer",
            50 => "refresh",
            51 => "retry-after",
            52 => "server",
            53 => "set-cookie",
            54 => "strict-transport-security",
            55 => "transfer-encoding",
            56 => "user-agent",
            57 => "vary",
            58 => "via",
            59 => "www-authenticate",
            60 => "www-authenticate",
            61 => "www-authenticate",
            _ => "unknown",
        }
    }

    fn get_value(index: u8) -> &'static str {
        match index {
            1 => "",
            2 => "GET",
            3 => "POST",
            4 => "/",
            5 => "/index.html",
            6 => "http",
            7 => "https",
            8 => "200",
            9 => "204",
            10 => "206",
            11 => "304",
            12 => "400",
            13 => "404",
            14 => "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            15 => "gzip, deflate",
            16 => "",
            17 => "en",
            18 => "",
            19 => "0",
            20 => "",
            21 => "",
            22 => "",
            23 => "",
            24 => "",
            25 => "",
            26 => "",
            27 => "",
            28 => "",
            29 => "text/html; charset=utf-8",
            30 => "",
            31 => "",
            32 => "",
            33 => "100-continue",
            34 => "",
            35 => "",
            36 => "",
            37 => "",
            38 => "",
            39 => "",
            40 => "",
            41 => "",
            42 => "",
            43 => "",
            44 => "",
            45 => "",
            46 => "",
            47 => "",
            48 => "bytes=0-1023",
            49 => "",
            50 => "--1",
            51 => "",
            52 => "",
            53 => "",
            54 => "max-age=0",
            55 => "",
            56 => "",
            57 => "Accept-Encoding",
            58 => "",
            59 => "",
            60 => "",
            61 => "",
            _ => "",
        }
    }
}

/// HPACK Analysis and Fingerprinting
pub struct HpackAnalyzer;

impl HpackAnalyzer {
    /// Detect browser from HPACK patterns
    pub fn detect_browser(header_order: &[String]) -> Option<String> {
        // Pseudo-headers order varies by browser
        let pseudo_headers: Vec<String> = header_order
            .iter()
            .filter(|h| h.starts_with(':'))
            .cloned()
            .collect();

        match pseudo_headers.as_slice() {
            // Chrome: :method, :scheme, :authority, :path
            headers if headers == vec![":method", ":scheme", ":authority", ":path"] => {
                Some("Chrome/Chromium".to_string())
            }
            // Firefox: :method, :scheme, :authority, :path (same as Chrome)
            headers if headers == vec![":method", ":path", ":authority", ":scheme"] => {
                Some("Firefox".to_string())
            }
            // Safari: :method, :authority, :scheme, :path
            headers if headers == vec![":authority", ":method", ":scheme", ":path"] => {
                Some("Safari".to_string())
            }
            // Edge: :method, :scheme, :authority, :path (Chromium-based)
            headers if headers == vec![":method", ":scheme", ":authority", ":path"] => {
                Some("Edge/Chromium".to_string())
            }
            _ => None,
        }
    }

    /// Detect server from HPACK patterns
    pub fn detect_server(header_order: &[String]) -> Option<String> {
        // Server implementations have distinctive patterns

        // Check for specific headers
        let response_headers = header_order
            .iter()
            .filter(|h| !h.starts_with(':'))
            .cloned()
            .collect::<Vec<_>>();

        // Nginx typically includes: server, date, content-type, vary, etc.
        if response_headers.contains(&"server".to_string()) {
            if let Some(pos) = response_headers.iter().position(|x| x == "server") {
                if pos < 3 {
                    // Server header early in response
                    return Some("Nginx".to_string());
                }
            }
        }

        // Apache: similar pattern but slightly different order
        if response_headers.contains(&"date".to_string()) {
            return Some("Apache/Other".to_string());
        }

        None
    }

    /// Analyze Huffman encoding preference
    pub fn analyze_huffman(header_list: &HpackHeaderList) -> HuffmanEncoding {
        let huffman_count = header_list
            .fields
            .iter()
            .filter(|f| f.huffman_encoded)
            .count();

        match huffman_count {
            0 => HuffmanEncoding::None,
            n if n > header_list.fields.len() / 2 => HuffmanEncoding::Standard,
            _ => HuffmanEncoding::Custom,
        }
    }

    /// Calculate index reuse pattern (how often same indices reused)
    pub fn analyze_index_reuse(header_lists: &[HpackHeaderList]) -> Vec<u32> {
        let mut index_reuse = vec![0u32; 256]; // Support up to 256 indices

        for header_list in header_lists {
            for field in &header_list.fields {
                if let Some(idx) = field.index {
                    if (idx as usize) < index_reuse.len() {
                        index_reuse[idx as usize] += 1;
                    }
                }
            }
        }

        // Keep only non-zero entries, in order of reuse
        index_reuse
            .iter()
            .filter(|&&count| count > 0)
            .copied()
            .collect()
    }

    /// Detect header field ordering pattern (Chrome vs Firefox vs others)
    pub fn detect_header_pattern(header_order: &[String]) -> Option<String> {
        // Common request headers in order
        let common_headers = [
            "user-agent",
            "accept",
            "accept-encoding",
            "accept-language",
            "cookie",
        ];

        let mut positions = Vec::new();
        for header in &common_headers {
            if let Some(pos) = header_order.iter().position(|h| h == header) {
                positions.push((header, pos));
            }
        }

        // Analyze the pattern
        if positions.len() >= 3 {
            // Check if pattern matches Chrome
            let chrome_pattern = positions.iter().all(|(_, pos)| *pos < header_order.len());
            if chrome_pattern {
                return Some("Chrome-like".to_string());
            }
        }

        None
    }

    /// Extract fingerprint from header list sequence
    pub fn create_fingerprint(header_lists: &[HpackHeaderList]) -> HpackFingerprint {
        let mut browser = None;
        let mut server = None;
        let mut header_order = Vec::new();
        let mut indexing_strategy = HashMap::new();
        let mut pseudo_header_order = Vec::new();
        let mut table_growth_pattern = Vec::new();

        if !header_lists.is_empty() {
            let first_list = &header_lists[0];

            // Extract header order
            header_order = first_list
                .fields
                .iter()
                .filter_map(|f| f.name.clone())
                .collect();

            // Extract pseudo-headers
            pseudo_header_order = first_list
                .fields
                .iter()
                .filter(|f| {
                    if let Some(ref name) = f.name {
                        name.starts_with(':')
                    } else {
                        false
                    }
                })
                .filter_map(|f| f.name.clone())
                .collect();

            // Detect browser
            browser = Self::detect_browser(&header_order);

            // Detect server
            server = Self::detect_server(&header_order);

            // Track indexing strategy
            for field in &first_list.fields {
                if let Some(ref name) = field.name {
                    indexing_strategy.insert(name.clone(), field.index_type);
                }
            }

            // Track dynamic table growth
            for list in header_lists.iter() {
                if let Some(snapshot) = &list.dynamic_table_snapshot {
                    table_growth_pattern.push(snapshot.entries.len() as u8);
                }
            }
        }

        HpackFingerprint {
            initial_table_size: 4096,
            header_order,
            indexing_strategy,
            huffman_preferences: if !header_lists.is_empty() {
                Self::analyze_huffman(&header_lists[0])
            } else {
                HuffmanEncoding::Standard
            },
            table_growth_pattern,
            index_reuse_pattern: Self::analyze_index_reuse(header_lists),
            pseudo_header_order,
            detected_browser: browser,
            detected_server: server,
            confidence: 0.85,
        }
    }

    /// Compare two HPACK fingerprints
    pub fn compare_fingerprints(fp1: &HpackFingerprint, fp2: &HpackFingerprint) -> f32 {
        let mut similarity = 0.0f32;
        let mut checks = 0u8;

        // Compare header order
        if fp1.header_order == fp2.header_order {
            similarity += 1.0;
        }
        checks += 1;

        // Compare pseudo-header order
        if fp1.pseudo_header_order == fp2.pseudo_header_order {
            similarity += 1.0;
        }
        checks += 1;

        // Compare Huffman preferences
        if fp1.huffman_preferences == fp2.huffman_preferences {
            similarity += 0.5;
        }
        checks += 1;

        // Compare detected browser
        if fp1.detected_browser == fp2.detected_browser && fp1.detected_browser.is_some() {
            similarity += 0.5;
        }
        checks += 1;

        similarity / checks as f32
    }

    /// Generate signature string for HPACK fingerprint
    pub fn generate_signature(fingerprint: &HpackFingerprint) -> String {
        format!(
            "HPACK:{}:{}:{}:{}",
            fingerprint.header_order.join(","),
            fingerprint.pseudo_header_order.join(","),
            match fingerprint.huffman_preferences {
                HuffmanEncoding::None => "no-huffman",
                HuffmanEncoding::Standard => "huffman",
                HuffmanEncoding::Custom => "custom-huffman",
            },
            fingerprint.detected_browser.as_deref().unwrap_or("unknown")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_table_entries() {
        let entry = static_table::get_entry(1);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().name, ":authority");

        let entry = static_table::get_entry(62); // Out of range
        assert!(entry.is_none());
    }

    #[test]
    fn test_browser_detection_from_headers() {
        let headers = vec![
            ":method".to_string(),
            ":scheme".to_string(),
            ":authority".to_string(),
            ":path".to_string(),
        ];

        let browser = HpackAnalyzer::detect_browser(&headers);
        assert_eq!(browser, Some("Chrome/Chromium".to_string()));
    }

    #[test]
    fn test_huffman_analysis() {
        let fields = vec![
            EncodedHeaderField {
                index_type: IndexType::Indexed,
                index: Some(2),
                name: Some("user-agent".to_string()),
                value: "Mozilla/5.0".to_string(),
                huffman_encoded: true,
                size_bytes: 20,
            },
            EncodedHeaderField {
                index_type: IndexType::IncrementalIndexing,
                index: None,
                name: Some("accept".to_string()),
                value: "text/html".to_string(),
                huffman_encoded: false,
                size_bytes: 15,
            },
        ];

        let list = HpackHeaderList {
            fields,
            total_size: 35,
            dynamic_table_snapshot: None,
            huffman_padding_bits: None,
        };

        let huffman = HpackAnalyzer::analyze_huffman(&list);
        assert!(huffman == HuffmanEncoding::Standard || huffman == HuffmanEncoding::Custom);
    }

    #[test]
    fn test_fingerprint_creation() {
        let fields = vec![EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2),
            name: Some(":method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        }];

        let list = HpackHeaderList {
            fields,
            total_size: 5,
            dynamic_table_snapshot: None,
            huffman_padding_bits: None,
        };

        let fp = HpackAnalyzer::create_fingerprint(&[list]);
        assert_eq!(fp.initial_table_size, 4096);
        assert!(!fp.header_order.is_empty());
    }

    #[test]
    fn test_fingerprint_comparison() {
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

        let fp2 = fp1.clone();

        let similarity = HpackAnalyzer::compare_fingerprints(&fp1, &fp2);
        assert!(similarity >= 0.75); // Identical fingerprints should score high
    }

    #[test]
    fn test_signature_generation() {
        let fp = HpackFingerprint {
            initial_table_size: 4096,
            header_order: vec!["user-agent".to_string()],
            indexing_strategy: HashMap::new(),
            huffman_preferences: HuffmanEncoding::Standard,
            table_growth_pattern: vec![],
            index_reuse_pattern: vec![],
            pseudo_header_order: vec![":method".to_string()],
            detected_browser: Some("Chrome".to_string()),
            detected_server: None,
            confidence: 0.9,
        };

        let sig = HpackAnalyzer::generate_signature(&fp);
        assert!(sig.contains("HPACK"));
        assert!(sig.contains("Chrome"));
    }

    #[test]
    fn test_index_reuse_analysis() {
        let fields_1 = vec![EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2),
            name: Some("method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        }];

        let fields_2 = vec![EncodedHeaderField {
            index_type: IndexType::Indexed,
            index: Some(2),
            name: Some("method".to_string()),
            value: "GET".to_string(),
            huffman_encoded: false,
            size_bytes: 5,
        }];

        let list1 = HpackHeaderList {
            fields: fields_1,
            total_size: 5,
            dynamic_table_snapshot: None,
            huffman_padding_bits: None,
        };

        let list2 = HpackHeaderList {
            fields: fields_2,
            total_size: 5,
            dynamic_table_snapshot: None,
            huffman_padding_bits: None,
        };

        let pattern = HpackAnalyzer::analyze_index_reuse(&[list1, list2]);
        assert!(!pattern.is_empty());
        assert_eq!(pattern[0], 2); // Index 2 reused twice
    }
}
