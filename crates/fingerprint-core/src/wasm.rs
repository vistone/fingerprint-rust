//! WebAssembly (WASM) Fingerprinting Detection
//!
//! Provides detection and analysis of WebAssembly modules and their characteristics
//! which can be used for browser fingerprinting and bot detection.
//!
//! Modern browsers expose WASM capabilities that can be fingerprinted through:
//! - WASM version and feature detection
//! - Memory/table limits and configurations
//! - Import/export patterns
//! - Module compilation behavior

use serde::{Deserialize, Serialize};

/// WebAssembly version support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmVersion {
    /// WASM MVP (Minimum Viable Product) - v1.0
    V1,
    /// WASM with threads and atomics
    Threads,
    /// WASM with SIMD
    Simd,
    /// WASM with reference types
    ReferenceTypes,
    /// WASM with bulk memory operations
    BulkMemory,
    /// WASM with multi-value
    MultiValue,
    /// WASM with tail calls
    TailCall,
    /// WASM with extended const expressions
    ExtendedConst,
    /// WASM with exception handling
    ExceptionHandling,
    /// WASM with garbage collection
    GarbageCollection,
}

impl WasmVersion {
    /// Get the feature name as string
    pub fn feature_name(&self) -> &'static str {
        match self {
            Self::V1 => "wasm_v1",
            Self::Threads => "wasm_threads",
            Self::Simd => "wasm_simd",
            Self::ReferenceTypes => "wasm_reference_types",
            Self::BulkMemory => "wasm_bulk_memory",
            Self::MultiValue => "wasm_multi_value",
            Self::TailCall => "wasm_tail_call",
            Self::ExtendedConst => "wasm_extended_const",
            Self::ExceptionHandling => "wasm_exception_handling",
            Self::GarbageCollection => "wasm_gc",
        }
    }
}

/// WebAssembly memory configuration fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmMemoryFingerprint {
    /// Initial memory size in pages (64KB each)
    pub initial_pages: u32,

    /// Maximum memory size in pages (if specified)
    pub maximum_pages: Option<u32>,

    /// Whether memory is shared (for threads)
    pub is_shared: bool,

    /// Memory growth pattern (linear, exponential, or fixed)
    pub growth_pattern: String,
}

impl WasmMemoryFingerprint {
    /// Create fingerprint for typical browser WASM memory
    pub fn browser_default() -> Self {
        Self {
            initial_pages: 256,      // 16MB
            maximum_pages: Some(256), // 16MB max (typical browser limit)
            is_shared: false,
            growth_pattern: "linear".to_string(),
        }
    }

    /// Create fingerprint for Node.js WASM memory
    pub fn nodejs_default() -> Self {
        Self {
            initial_pages: 256,
            maximum_pages: Some(65536), // 4GB max
            is_shared: false,
            growth_pattern: "exponential".to_string(),
        }
    }

    /// Generate a fingerprint string
    pub fn fingerprint(&self) -> String {
        format!(
            "wasm_mem_{}_{}_{}",
            self.initial_pages,
            self.maximum_pages.map(|m| m.to_string()).unwrap_or_else(|| "unlimited".to_string()),
            if self.is_shared { "shared" } else { "private" }
        )
    }
}

/// WebAssembly table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTableFingerprint {
    /// Initial table size
    pub initial_size: u32,

    /// Maximum table size (if specified)
    pub maximum_size: Option<u32>,

    /// Element type (funcref, externref, etc.)
    pub element_type: String,
}

/// WebAssembly capabilities and features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCapabilities {
    /// Supported WASM versions/features
    pub versions: Vec<WasmVersion>,

    /// Memory configuration fingerprint
    pub memory: WasmMemoryFingerprint,

    /// Table configuration fingerprint
    pub table: Option<WasmTableFingerprint>,

    /// Whether streaming compilation is supported
    pub streaming_compilation: bool,

    /// Whether instantiate streaming is supported
    pub instantiate_streaming: bool,

    /// Maximum module size that can be compiled (bytes)
    pub max_module_size: Option<u64>,

    /// Compilation speed estimate (modules per second)
    pub compilation_speed: Option<f64>,

    /// Whether WASM is available
    pub available: bool,
}

impl WasmCapabilities {
    /// Create capabilities for no WASM support
    pub fn none() -> Self {
        Self {
            versions: Vec::new(),
            memory: WasmMemoryFingerprint::browser_default(),
            table: None,
            streaming_compilation: false,
            instantiate_streaming: false,
            max_module_size: None,
            compilation_speed: None,
            available: false,
        }
    }

    /// Create default capabilities for modern browsers
    pub fn modern_browser() -> Self {
        Self {
            versions: vec![
                WasmVersion::V1,
                WasmVersion::Threads,
                WasmVersion::Simd,
                WasmVersion::ReferenceTypes,
                WasmVersion::BulkMemory,
                WasmVersion::MultiValue,
            ],
            memory: WasmMemoryFingerprint::browser_default(),
            table: Some(WasmTableFingerprint {
                initial_size: 1,
                maximum_size: None,
                element_type: "funcref".to_string(),
            }),
            streaming_compilation: true,
            instantiate_streaming: true,
            max_module_size: Some(1024 * 1024 * 100), // 100MB
            compilation_speed: Some(10.0),              // 10 modules/sec
            available: true,
        }
    }

    /// Create capabilities for Node.js environment
    pub fn nodejs() -> Self {
        Self {
            versions: vec![
                WasmVersion::V1,
                WasmVersion::Threads,
                WasmVersion::Simd,
                WasmVersion::ReferenceTypes,
                WasmVersion::BulkMemory,
                WasmVersion::MultiValue,
                WasmVersion::TailCall,
            ],
            memory: WasmMemoryFingerprint::nodejs_default(),
            table: Some(WasmTableFingerprint {
                initial_size: 1,
                maximum_size: None,
                element_type: "funcref".to_string(),
            }),
            streaming_compilation: true,
            instantiate_streaming: true,
            max_module_size: None, // Virtually unlimited
            compilation_speed: Some(50.0), // Much faster than browser
            available: true,
        }
    }

    /// Generate a fingerprint string for these capabilities
    pub fn fingerprint(&self) -> String {
        if !self.available {
            return "wasm_none".to_string();
        }

        use sha2::{Digest, Sha256};

        let features = self
            .versions
            .iter()
            .map(|v| v.feature_name())
            .collect::<Vec<_>>()
            .join(",");

        let mut hasher = Sha256::new();
        hasher.update(features.as_bytes());
        hasher.update(self.memory.fingerprint().as_bytes());
        hasher.update(if self.streaming_compilation { "stream_yes" } else { "stream_no" }.as_bytes());

        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);

        format!(
            "wasm_{:02}_{}_{}",
            self.versions.len(),
            if self.streaming_compilation { "s" } else { "n" },
            &hash_hex[0..12]
        )
    }

    /// Check if capabilities match expected browser behavior
    pub fn matches_browser(
        &self,
        browser: &str,
        version: u32,
    ) -> Result<(), String> {
        match browser.to_lowercase().as_str() {
            "chrome" | "chromium" | "edge" => {
                if version < 69 {
                    if self.available {
                        return Err(format!(
                            "{} v{} should not have full WASM support",
                            browser, version
                        ));
                    }
                } else {
                    if !self.available {
                        return Err(format!(
                            "{} v{} should have WASM support",
                            browser, version
                        ));
                    }

                    // Chrome 69+ should support threads
                    if version >= 74 && !self.versions.contains(&WasmVersion::Threads) {
                        return Err(format!(
                            "{} v{} should support WASM threads",
                            browser, version
                        ));
                    }

                    // Chrome 91+ should support SIMD
                    if version >= 91 && !self.versions.contains(&WasmVersion::Simd) {
                        return Err(format!(
                            "{} v{} should support WASM SIMD",
                            browser, version
                        ));
                    }
                }
            }
            "firefox" => {
                if version < 52 {
                    if self.available {
                        return Err(format!(
                            "{} v{} should not have WASM support",
                            browser, version
                        ));
                    }
                } else {
                    if !self.available {
                        return Err(format!(
                            "{} v{} should have WASM support",
                            browser, version
                        ));
                    }

                    // Firefox 79+ should support threads
                    if version >= 79 && !self.versions.contains(&WasmVersion::Threads) {
                        return Err(format!(
                            "{} v{} should support WASM threads",
                            browser, version
                        ));
                    }
                }
            }
            "safari" => {
                if version < 11 {
                    if self.available {
                        return Err(format!(
                            "{} v{} should not have WASM support",
                            browser, version
                        ));
                    }
                } else {
                    if !self.available {
                        return Err(format!(
                            "{} v{} should have WASM support",
                            browser, version
                        ));
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Browser-specific WASM support database
pub struct WasmBrowserSupport;

impl WasmBrowserSupport {
    /// Check if a browser version supports WASM
    pub fn supports_wasm(browser: &str, version: u32) -> bool {
        match browser.to_lowercase().as_str() {
            "chrome" | "chromium" | "edge" => version >= 57,
            "firefox" => version >= 52,
            "safari" => version >= 11,
            "opera" => version >= 44,
            _ => false,
        }
    }

    /// Check if a browser version supports WASM threads
    pub fn supports_threads(browser: &str, version: u32) -> bool {
        match browser.to_lowercase().as_str() {
            "chrome" | "chromium" | "edge" => version >= 74,
            "firefox" => version >= 79,
            "safari" => version >= 14, // Safari 14.1+
            _ => false,
        }
    }

    /// Check if a browser version supports WASM SIMD
    pub fn supports_simd(browser: &str, version: u32) -> bool {
        match browser.to_lowercase().as_str() {
            "chrome" | "chromium" | "edge" => version >= 91,
            "firefox" => version >= 89,
            "safari" => version >= 16, // Safari 16.4+
            _ => false,
        }
    }

    /// Get expected capabilities for a browser
    pub fn expected_capabilities(browser: &str, version: u32) -> WasmCapabilities {
        if !Self::supports_wasm(browser, version) {
            return WasmCapabilities::none();
        }

        let mut capabilities = WasmCapabilities::modern_browser();
        capabilities.versions = vec![WasmVersion::V1];

        if Self::supports_threads(browser, version) {
            capabilities.versions.push(WasmVersion::Threads);
        }

        if Self::supports_simd(browser, version) {
            capabilities.versions.push(WasmVersion::Simd);
        }

        // Add other common features for modern browsers
        if version >= 85 {
            capabilities.versions.push(WasmVersion::ReferenceTypes);
            capabilities.versions.push(WasmVersion::BulkMemory);
            capabilities.versions.push(WasmVersion::MultiValue);
        }

        capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_version_features() {
        assert_eq!(WasmVersion::V1.feature_name(), "wasm_v1");
        assert_eq!(WasmVersion::Threads.feature_name(), "wasm_threads");
        assert_eq!(WasmVersion::Simd.feature_name(), "wasm_simd");
    }

    #[test]
    fn test_memory_fingerprint() {
        let mem = WasmMemoryFingerprint::browser_default();
        assert_eq!(mem.initial_pages, 256);
        assert_eq!(mem.maximum_pages, Some(256));
        assert!(!mem.is_shared);

        let fp = mem.fingerprint();
        assert!(fp.contains("256"));
        assert!(fp.contains("private"));
    }

    #[test]
    fn test_wasm_capabilities_none() {
        let caps = WasmCapabilities::none();
        assert!(!caps.available);
        assert_eq!(caps.versions.len(), 0);
        assert_eq!(caps.fingerprint(), "wasm_none");
    }

    #[test]
    fn test_wasm_capabilities_modern_browser() {
        let caps = WasmCapabilities::modern_browser();
        assert!(caps.available);
        assert!(caps.versions.contains(&WasmVersion::V1));
        assert!(caps.versions.contains(&WasmVersion::Simd));
        assert!(caps.streaming_compilation);
    }

    #[test]
    fn test_wasm_capabilities_nodejs() {
        let caps = WasmCapabilities::nodejs();
        assert!(caps.available);
        assert!(caps.versions.contains(&WasmVersion::TailCall));
        assert!(caps.compilation_speed.unwrap() > 40.0);
    }

    #[test]
    fn test_browser_support() {
        assert!(WasmBrowserSupport::supports_wasm("chrome", 100));
        assert!(!WasmBrowserSupport::supports_wasm("chrome", 50));

        assert!(WasmBrowserSupport::supports_threads("firefox", 100));
        assert!(!WasmBrowserSupport::supports_threads("firefox", 70));

        assert!(WasmBrowserSupport::supports_simd("chrome", 100));
        assert!(!WasmBrowserSupport::supports_simd("chrome", 80));
    }

    #[test]
    fn test_expected_capabilities() {
        let caps = WasmBrowserSupport::expected_capabilities("chrome", 120);
        assert!(caps.available);
        assert!(caps.versions.contains(&WasmVersion::Threads));
        assert!(caps.versions.contains(&WasmVersion::Simd));
    }

    #[test]
    fn test_capabilities_matching() {
        let caps = WasmCapabilities::modern_browser();
        assert!(caps.matches_browser("chrome", 120).is_ok());

        let no_caps = WasmCapabilities::none();
        assert!(no_caps.matches_browser("chrome", 120).is_err());
    }

    #[test]
    fn test_fingerprint_generation() {
        let caps = WasmCapabilities::modern_browser();
        let fp = caps.fingerprint();

        assert!(fp.starts_with("wasm_"));
        assert!(fp.contains("_s_")); // streaming enabled
    }
}
