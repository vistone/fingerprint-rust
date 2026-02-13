use serde::{Deserialize, Serialize};
/// Browser Version Management and Registry
///
/// Provides automatic version handling for browser fingerprints:
/// - Version registry with release tracking
/// - Automatic migration from old to new versions
/// - Version range mapping
/// - Rapid browser version adaptation
use std::collections::{BTreeMap, HashMap};

/// Browser type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Opera,
}

impl std::fmt::Display for BrowserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chrome => write!(f, "Chrome"),
            Self::Firefox => write!(f, "Firefox"),
            Self::Safari => write!(f, "Safari"),
            Self::Edge => write!(f, "Edge"),
            Self::Opera => write!(f, "Opera"),
        }
    }
}

/// Version entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Browser version number
    pub version: u32,
    /// Release date (YYYY-MM-DD format)
    pub release_date: String,
    /// TLS 1.3 support status
    pub tls13_support: bool,
    /// ECH (RFC 9180) support
    pub ech_support: bool,
    /// HTTP/2 support
    pub http2_support: bool,
    /// HTTP/3 (QUIC) support
    pub http3_support: bool,
    /// PSK (session resumption) support
    pub psk_support: bool,
    /// 0-RTT (Early Data) support
    pub early_data_support: bool,
    /// Post-quantum hybrid KEMs support (Kyber768, etc.)
    pub pq_support: bool,
    /// Brotli compression support
    pub brotli_support: bool,
    /// Previous compatible version for migration
    pub fallback_version: Option<u32>,
    /// Profile function name
    pub profile_fn: String,
    /// Remarks or special features
    pub remarks: Option<String>,
}

/// Browser version registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistry {
    /// Chrome versions registry
    pub chrome: BTreeMap<u32, VersionEntry>,
    /// Firefox versions registry
    pub firefox: BTreeMap<u32, VersionEntry>,
    /// Safari versions registry
    pub safari: BTreeMap<u32, VersionEntry>,
    /// Edge versions registry
    pub edge: BTreeMap<u32, VersionEntry>,
    /// Opera versions registry
    pub opera: BTreeMap<u32, VersionEntry>,
}

impl VersionRegistry {
    /// Create a new version registry with current browser versions
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            chrome: BTreeMap::new(),
            firefox: BTreeMap::new(),
            safari: BTreeMap::new(),
            edge: BTreeMap::new(),
            opera: BTreeMap::new(),
        };

        registry.init_chrome_versions();
        registry.init_firefox_versions();
        registry.init_safari_versions();
        registry.init_edge_versions();
        registry.init_opera_versions();

        registry
    }

    /// Initialize Chrome versions registry
    fn init_chrome_versions(&mut self) {
        // Chrome 103-112: Basic versions
        self.add_chrome_version(
            103,
            "2022-09-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            None,
            "chrome_103",
        );
        self.add_chrome_version(
            104,
            "2022-10-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(103),
            "chrome_104",
        );
        self.add_chrome_version(
            105,
            "2022-11-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(104),
            "chrome_105",
        );
        self.add_chrome_version(
            106,
            "2022-12-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(105),
            "chrome_106",
        );
        self.add_chrome_version(
            107,
            "2023-01-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(106),
            "chrome_107",
        );
        self.add_chrome_version(
            108,
            "2023-02-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(107),
            "chrome_108",
        );
        self.add_chrome_version(
            109,
            "2023-03-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(108),
            "chrome_109",
        );
        self.add_chrome_version(
            110,
            "2023-04-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(109),
            "chrome_110",
        );
        self.add_chrome_version(
            111,
            "2023-05-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(110),
            "chrome_111",
        );
        self.add_chrome_version(
            112,
            "2023-06-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(111),
            "chrome_112",
        );

        // Chrome 116: Special PSK variants
        self.add_chrome_version(
            116,
            "2023-09-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(112),
            "chrome_116_psk",
        );
        // Note: chrome_116_psk_pq would need separate registration if needed

        // Chrome 117: Return to standard versions
        self.add_chrome_version(
            117,
            "2023-10-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(116),
            "chrome_117",
        );

        // Chrome 120-138: Modern versions
        self.add_chrome_version(
            120,
            "2024-01-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(117),
            "chrome_120",
        );
        self.add_chrome_version(
            121,
            "2024-02-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(120),
            "chrome_121",
        );
        self.add_chrome_version(
            122,
            "2024-03-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(121),
            "chrome_122",
        );
        self.add_chrome_version(
            123,
            "2024-04-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(122),
            "chrome_123",
        );
        self.add_chrome_version(
            124,
            "2024-05-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(123),
            "chrome_124",
        );
        self.add_chrome_version(
            125,
            "2024-06-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(124),
            "chrome_125",
        );
        self.add_chrome_version(
            126,
            "2024-07-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(125),
            "chrome_126",
        );
        self.add_chrome_version(
            127,
            "2024-08-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(126),
            "chrome_127",
        );
        self.add_chrome_version(
            128,
            "2024-09-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(127),
            "chrome_128",
        );
        self.add_chrome_version(
            129,
            "2024-10-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(128),
            "chrome_129",
        );
        self.add_chrome_version(
            130,
            "2024-11-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(129),
            "chrome_130",
        );
        self.add_chrome_version(
            131,
            "2024-12-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(130),
            "chrome_131",
        );
        self.add_chrome_version(
            132,
            "2025-01-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(131),
            "chrome_132",
        );
        self.add_chrome_version(
            133,
            "2025-02-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(132),
            "chrome_133",
        );
        self.add_chrome_version(
            134,
            "2025-03-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(133),
            "chrome_134",
        );
        self.add_chrome_version(
            135,
            "2025-04-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(134),
            "chrome_135",
        );
        self.add_chrome_version(
            136,
            "2025-05-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(135),
            "chrome_136",
        );
        self.add_chrome_version(
            137,
            "2025-06-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(136),
            "chrome_137",
        );
        self.add_chrome_version(
            138,
            "2025-07-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(137),
            "chrome_138",
        );

        // Special variants
        self.add_chrome_version(
            1331,
            "2025-02-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(133),
            "chrome_133_psk",
        );
        self.add_chrome_version(
            1332,
            "2025-02-01",
            true,
            true,
            true,
            true,
            false,
            true,
            false,
            Some(133),
            "chrome_133_0rtt",
        );
        self.add_chrome_version(
            1333,
            "2025-02-01",
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            Some(133),
            "chrome_133_psk_0rtt",
        );
    }

    /// Initialize Firefox versions registry
    fn init_firefox_versions(&mut self) {
        // Firefox 102-123: Older versions
        self.add_firefox_version(
            102,
            "2022-09-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            None,
            "firefox_102",
        );
        self.add_firefox_version(
            104,
            "2022-11-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(102),
            "firefox_104",
        );
        self.add_firefox_version(
            105,
            "2022-12-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(104),
            "firefox_105",
        );
        self.add_firefox_version(
            106,
            "2023-01-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(105),
            "firefox_106",
        );
        self.add_firefox_version(
            108,
            "2023-03-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(106),
            "firefox_108",
        );
        self.add_firefox_version(
            110,
            "2023-05-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(108),
            "firefox_110",
        );
        self.add_firefox_version(
            117,
            "2023-12-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(110),
            "firefox_117",
        );
        self.add_firefox_version(
            120,
            "2024-03-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(117),
            "firefox_120",
        );
        self.add_firefox_version(
            123,
            "2024-06-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(120),
            "firefox_123",
        );

        // Firefox 130-138: Modern versions
        self.add_firefox_version(
            130,
            "2024-09-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(123),
            "firefox_130",
        );
        self.add_firefox_version(
            131,
            "2024-10-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(130),
            "firefox_131",
        );
        self.add_firefox_version(
            132,
            "2024-11-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(131),
            "firefox_132",
        );
        self.add_firefox_version(
            133,
            "2024-12-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(132),
            "firefox_133",
        );
        self.add_firefox_version(
            134,
            "2025-01-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(133),
            "firefox_134",
        );
        self.add_firefox_version(
            135,
            "2025-02-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(134),
            "firefox_135",
        );
        self.add_firefox_version(
            136,
            "2025-03-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(135),
            "firefox_136",
        );
        self.add_firefox_version(
            137,
            "2025-04-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(136),
            "firefox_137",
        );
        self.add_firefox_version(
            138,
            "2025-05-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(137),
            "firefox_138",
        );
    }

    /// Initialize Safari versions registry
    fn init_safari_versions(&mut self) {
        // Safari desktop versions
        self.add_safari_version(
            150,
            "2021-09-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            None,
            "safari_15_0",
        );
        self.add_safari_version(
            156,
            "2022-03-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(150),
            "safari_15_6_1",
        );
        self.add_safari_version(
            157,
            "2022-06-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(156),
            "safari_15_7",
        );
        self.add_safari_version(
            160,
            "2022-09-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(157),
            "safari_16_0",
        );
        self.add_safari_version(
            170,
            "2023-09-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(160),
            "safari_17_0",
        );
        self.add_safari_version(
            175,
            "2023-12-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(170),
            "safari_17_5",
        );
        self.add_safari_version(
            180,
            "2024-09-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(175),
            "safari_18_0",
        );
        self.add_safari_version(
            181,
            "2024-10-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(180),
            "safari_18_1",
        );
        self.add_safari_version(
            182,
            "2024-11-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(181),
            "safari_18_2",
        );
        self.add_safari_version(
            183,
            "2024-12-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(182),
            "safari_18_3",
        );

        // Safari iOS versions
        self.add_safari_version(
            1550,
            "2022-01-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(150),
            "safari_ios_15_5",
        );
        self.add_safari_version(
            1560,
            "2022-03-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(1550),
            "safari_ios_15_6",
        );
        self.add_safari_version(
            1600,
            "2022-09-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(1560),
            "safari_ios_16_0",
        );
        self.add_safari_version(
            1700,
            "2023-09-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(1600),
            "safari_ios_17_0",
        );
        self.add_safari_version(
            1800,
            "2024-09-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(1700),
            "safari_ios_18_0",
        );
        self.add_safari_version(
            1810,
            "2024-10-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(1800),
            "safari_ios_18_1",
        );
        self.add_safari_version(
            1820,
            "2024-11-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(1810),
            "safari_ios_18_2",
        );
        self.add_safari_version(
            1830,
            "2024-12-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(1820),
            "safari_ios_18_3",
        );
        self.add_safari_version(
            1850,
            "2025-02-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(1830),
            "safari_ios_18_5",
        );

        // Safari iPad versions
        self.add_safari_version(
            15600,
            "2022-03-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(150),
            "safari_ipad_15_6",
        );
    }

    /// Initialize Edge versions registry
    fn init_edge_versions(&mut self) {
        self.add_edge_version(
            120,
            "2024-01-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            None,
            "edge_120",
        );
        self.add_edge_version(
            124,
            "2024-05-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(120),
            "edge_124",
        );
        self.add_edge_version(
            125,
            "2024-06-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(124),
            "edge_125",
        );
        self.add_edge_version(
            126,
            "2024-07-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(125),
            "edge_126",
        );
        self.add_edge_version(
            130,
            "2024-11-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(126),
            "edge_130",
        );
        self.add_edge_version(
            131,
            "2024-12-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(130),
            "edge_131",
        );
        self.add_edge_version(
            132,
            "2025-01-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(131),
            "edge_132",
        );
        self.add_edge_version(
            133,
            "2025-02-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(132),
            "edge_133",
        );
        self.add_edge_version(
            134,
            "2025-03-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(133),
            "edge_134",
        );
        self.add_edge_version(
            135,
            "2025-04-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(134),
            "edge_135",
        );
        self.add_edge_version(
            137,
            "2025-06-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(135),
            "edge_137",
        );
    }

    /// Initialize Opera versions registry
    fn init_opera_versions(&mut self) {
        self.add_opera_version(
            89,
            "2023-09-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            None,
            "opera_89",
        );
        self.add_opera_version(
            90,
            "2023-10-01",
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            Some(89),
            "opera_90",
        );
        self.add_opera_version(
            91,
            "2023-11-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(90),
            "opera_91",
        );
        self.add_opera_version(
            92,
            "2023-12-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(91),
            "opera_92",
        );
        self.add_opera_version(
            93,
            "2024-01-01",
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            Some(92),
            "opera_93",
        );
        self.add_opera_version(
            94,
            "2024-02-01",
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            Some(93),
            "opera_94",
        );
    }

    /// Add Chrome version entry
    fn add_chrome_version(
        &mut self,
        version: u32,
        release_date: &str,
        tls13: bool,
        ech: bool,
        http2: bool,
        http3: bool,
        psk: bool,
        early_data: bool,
        pq: bool,
        fallback: Option<u32>,
        profile_fn: &str,
    ) {
        self.chrome.insert(
            version,
            VersionEntry {
                version,
                release_date: release_date.to_string(),
                tls13_support: tls13,
                ech_support: ech,
                http2_support: http2,
                http3_support: http3,
                psk_support: psk,
                early_data_support: early_data,
                pq_support: pq,
                brotli_support: true,
                fallback_version: fallback,
                profile_fn: profile_fn.to_string(),
                remarks: None,
            },
        );
    }

    /// Add Firefox version entry
    fn add_firefox_version(
        &mut self,
        version: u32,
        release_date: &str,
        tls13: bool,
        ech: bool,
        http2: bool,
        http3: bool,
        psk: bool,
        early_data: bool,
        pq: bool,
        fallback: Option<u32>,
        profile_fn: &str,
    ) {
        self.firefox.insert(
            version,
            VersionEntry {
                version,
                release_date: release_date.to_string(),
                tls13_support: tls13,
                ech_support: ech,
                http2_support: http2,
                http3_support: http3,
                psk_support: psk,
                early_data_support: early_data,
                pq_support: pq,
                brotli_support: true,
                fallback_version: fallback,
                profile_fn: profile_fn.to_string(),
                remarks: None,
            },
        );
    }

    /// Add Safari version entry
    fn add_safari_version(
        &mut self,
        version: u32,
        release_date: &str,
        tls13: bool,
        ech: bool,
        http2: bool,
        http3: bool,
        psk: bool,
        early_data: bool,
        pq: bool,
        fallback: Option<u32>,
        profile_fn: &str,
    ) {
        self.safari.insert(
            version,
            VersionEntry {
                version,
                release_date: release_date.to_string(),
                tls13_support: tls13,
                ech_support: ech,
                http2_support: http2,
                http3_support: http3,
                psk_support: psk,
                early_data_support: early_data,
                pq_support: pq,
                brotli_support: true,
                fallback_version: fallback,
                profile_fn: profile_fn.to_string(),
                remarks: None,
            },
        );
    }

    /// Add Edge version entry
    fn add_edge_version(
        &mut self,
        version: u32,
        release_date: &str,
        tls13: bool,
        ech: bool,
        http2: bool,
        http3: bool,
        psk: bool,
        early_data: bool,
        pq: bool,
        fallback: Option<u32>,
        profile_fn: &str,
    ) {
        self.edge.insert(
            version,
            VersionEntry {
                version,
                release_date: release_date.to_string(),
                tls13_support: tls13,
                ech_support: ech,
                http2_support: http2,
                http3_support: http3,
                psk_support: psk,
                early_data_support: early_data,
                pq_support: pq,
                brotli_support: true,
                fallback_version: fallback,
                profile_fn: profile_fn.to_string(),
                remarks: None,
            },
        );
    }

    /// Add Opera version entry
    fn add_opera_version(
        &mut self,
        version: u32,
        release_date: &str,
        tls13: bool,
        ech: bool,
        http2: bool,
        http3: bool,
        psk: bool,
        early_data: bool,
        pq: bool,
        fallback: Option<u32>,
        profile_fn: &str,
    ) {
        self.opera.insert(
            version,
            VersionEntry {
                version,
                release_date: release_date.to_string(),
                tls13_support: tls13,
                ech_support: ech,
                http2_support: http2,
                http3_support: http3,
                psk_support: psk,
                early_data_support: early_data,
                pq_support: pq,
                brotli_support: true,
                fallback_version: fallback,
                profile_fn: profile_fn.to_string(),
                remarks: None,
            },
        );
    }

    /// Get version entry for a browser
    pub fn get_version(&self, browser: BrowserType, version: u32) -> Option<&VersionEntry> {
        match browser {
            BrowserType::Chrome => self.chrome.get(&version),
            BrowserType::Firefox => self.firefox.get(&version),
            BrowserType::Safari => self.safari.get(&version),
            BrowserType::Edge => self.edge.get(&version),
            BrowserType::Opera => self.opera.get(&version),
        }
    }

    /// Find nearest compatible version (for fallback)
    pub fn find_nearest_compatible(&self, browser: BrowserType, version: u32) -> Option<u32> {
        let versions = match browser {
            BrowserType::Chrome => &self.chrome,
            BrowserType::Firefox => &self.firefox,
            BrowserType::Safari => &self.safari,
            BrowserType::Edge => &self.edge,
            BrowserType::Opera => &self.opera,
        };

        // First check if exact version exists
        if versions.contains_key(&version) {
            return Some(version);
        }

        // Find nearest lower version
        let mut nearest = None;
        for (v, _) in versions.iter().rev() {
            if *v < version {
                nearest = Some(*v);
                break;
            }
        }

        nearest.or_else(|| versions.keys().next_back().copied())
    }

    /// Get latest version for a browser
    pub fn get_latest(&self, browser: BrowserType) -> Option<&VersionEntry> {
        let versions = match browser {
            BrowserType::Chrome => &self.chrome,
            BrowserType::Firefox => &self.firefox,
            BrowserType::Safari => &self.safari,
            BrowserType::Edge => &self.edge,
            BrowserType::Opera => &self.opera,
        };

        versions.iter().rev().next().map(|(_, entry)| entry)
    }

    /// Get all supported versions for a browser
    pub fn get_all_versions(&self, browser: BrowserType) -> Vec<(&u32, &VersionEntry)> {
        let versions = match browser {
            BrowserType::Chrome => &self.chrome,
            BrowserType::Firefox => &self.firefox,
            BrowserType::Safari => &self.safari,
            BrowserType::Edge => &self.edge,
            BrowserType::Opera => &self.opera,
        };

        versions.iter().collect()
    }

    /// Get versions with specific feature support
    pub fn get_with_feature(
        &self,
        browser: BrowserType,
        feature: &str,
    ) -> Vec<(&u32, &VersionEntry)> {
        self.get_all_versions(browser)
            .into_iter()
            .filter(|(_, entry)| match feature {
                "ech" => entry.ech_support,
                "http3" => entry.http3_support,
                "psk" => entry.psk_support,
                "early_data" => entry.early_data_support,
                "pq" => entry.pq_support,
                _ => false,
            })
            .collect()
    }

    /// Generate version migration mapping (for profile updates)
    pub fn get_migration_map(&self, browser: BrowserType) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let versions = match browser {
            BrowserType::Chrome => &self.chrome,
            BrowserType::Firefox => &self.firefox,
            BrowserType::Safari => &self.safari,
            BrowserType::Edge => &self.edge,
            BrowserType::Opera => &self.opera,
        };

        for (version, entry) in versions {
            let key = match browser {
                BrowserType::Chrome => format!("chrome_{}", version),
                BrowserType::Firefox => format!("firefox_{}", version),
                BrowserType::Safari => format!("safari_{}", version),
                BrowserType::Edge => format!("edge_{}", version),
                BrowserType::Opera => format!("opera_{}", version),
            };

            map.insert(key, entry.profile_fn.clone());
        }

        map
    }
}

impl Default for VersionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_registry_creation() {
        let registry = VersionRegistry::new();
        assert!(!registry.chrome.is_empty());
        assert!(!registry.firefox.is_empty());
    }

    #[test]
    fn test_get_version() {
        let registry = VersionRegistry::new();
        let entry = registry.get_version(BrowserType::Chrome, 133);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().profile_fn, "chrome_133");
    }

    #[test]
    fn test_find_nearest_compatible() {
        let registry = VersionRegistry::new();
        let nearest = registry.find_nearest_compatible(BrowserType::Chrome, 199);
        assert!(nearest.is_some());
    }

    #[test]
    fn test_get_latest() {
        let registry = VersionRegistry::new();
        let latest = registry.get_latest(BrowserType::Chrome);
        assert!(latest.is_some());
        assert!(latest.unwrap().version >= 130);
    }

    #[test]
    fn test_migration_map() {
        let registry = VersionRegistry::new();
        let map = registry.get_migration_map(BrowserType::Chrome);
        assert!(!map.is_empty());
        assert!(map.contains_key("chrome_133"));
    }
}
