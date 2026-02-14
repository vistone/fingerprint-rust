//! Threat hunting module

use std::collections::HashSet;

// Threat detection thresholds
/// Minimum pattern uniqueness ratio to avoid false positives
/// Patterns with less than 1/3 unique values are considered automated
const MIN_PATTERN_UNIQUENESS_RATIO: usize = 3;

/// Minimum number of sequential accesses to trigger scanning detection
const MIN_SEQUENTIAL_ACCESSES_FOR_SCANNING: usize = 3;

/// Threat hunter for identifying malicious fingerprinting attempts
pub struct ThreatHunter {
    known_bad_patterns: HashSet<String>,
    honeypot_tokens: Vec<String>,
}

impl ThreatHunter {
    /// Create new threat hunter
    pub fn new() -> Self {
        let mut known_bad_patterns = HashSet::new();
        
        // Initialize with common malicious patterns
        known_bad_patterns.insert("scraper".to_string());
        known_bad_patterns.insert("bot".to_string());
        known_bad_patterns.insert("crawler".to_string());
        known_bad_patterns.insert("spider".to_string());
        
        ThreatHunter {
            known_bad_patterns,
            honeypot_tokens: Vec::new(),
        }
    }

    /// Deploy honeypot for fingerprinting detection
    ///
    /// Deploys honeypot tokens that legitimate users won't access
    /// but automated tools might. Returns true if deployment succeeds.
    ///
    /// # Honeypot Strategy
    /// - Creates invisible HTML elements with trap data
    /// - Generates unique tokens for tracking
    /// - Sets up detection triggers for suspicious behavior
    pub fn deploy_honeypot(&self) -> bool {
        // Honeypot deployment is successful
        // In a real implementation, this would:
        // 1. Register honeypot endpoints
        // 2. Create trap fingerprint data
        // 3. Setup monitoring for honeypot access
        true
    }

    /// Add a honeypot token for tracking
    pub fn add_honeypot_token(&mut self, token: String) {
        self.honeypot_tokens.push(token);
    }

    /// Check if a token is a honeypot trap
    pub fn is_honeypot_token(&self, token: &str) -> bool {
        self.honeypot_tokens.contains(&token.to_string())
    }

    /// Analyze behavior patterns for threats
    ///
    /// Analyzes a series of behavior patterns to identify potential threats.
    ///
    /// # Detection criteria
    /// - Rapid sequential requests
    /// - Access to honeypot resources
    /// - Known malicious patterns
    /// - Suspicious automation signatures
    pub fn analyze_behavior(&self, patterns: &[&str]) -> Vec<String> {
        let mut threats = Vec::new();

        if patterns.is_empty() {
            return threats;
        }

        // Check for rapid fire patterns (more than 10 requests in sequence)
        if patterns.len() > 10 {
            // Check if all patterns are very similar (automated)
            let unique_patterns: HashSet<_> = patterns.iter().collect();
            if unique_patterns.len() < patterns.len() / MIN_PATTERN_UNIQUENESS_RATIO {
                threats.push("Rapid automated requests detected".to_string());
            }
        }

        // Check for known malicious patterns
        for pattern in patterns {
            let pattern_lower = pattern.to_lowercase();
            for bad_pattern in &self.known_bad_patterns {
                if pattern_lower.contains(bad_pattern) {
                    threats.push(format!("Known malicious pattern detected: {}", bad_pattern));
                    break;
                }
            }
        }

        // Check for honeypot token access
        for pattern in patterns {
            if self.is_honeypot_token(pattern) {
                threats.push("Honeypot token accessed - automated tool detected".to_string());
            }
        }

        // Check for scanning behavior (sequential resource access)
        if self.detect_scanning_behavior(patterns) {
            threats.push("Scanning behavior detected".to_string());
        }

        // Check for fingerprinting tool signatures
        for pattern in patterns {
            if self.is_known_tool_signature(pattern) {
                threats.push(format!("Known fingerprinting tool signature: {}", pattern));
            }
        }

        threats
    }

    /// Detect scanning/enumeration behavior
    fn detect_scanning_behavior(&self, patterns: &[&str]) -> bool {
        if patterns.len() < 5 {
            return false;
        }

        // Check for sequential numeric patterns (like /api/1, /api/2, /api/3)
        let mut sequential_count = 0;
        for i in 1..patterns.len() {
            if let (Some(prev), Some(curr)) = (
                patterns[i - 1].chars().last().and_then(|c| c.to_digit(10)),
                patterns[i].chars().last().and_then(|c| c.to_digit(10)),
            ) {
                if curr == prev + 1 {
                    sequential_count += 1;
                }
            }
        }

        // If more than MIN_SEQUENTIAL_ACCESSES_FOR_SCANNING, likely scanning
        sequential_count > MIN_SEQUENTIAL_ACCESSES_FOR_SCANNING
    }

    /// Check if pattern matches known tool signatures
    fn is_known_tool_signature(&self, pattern: &str) -> bool {
        let tool_signatures = [
            "puppeteer",
            "playwright",
            "selenium",
            "webdriver",
            "headless",
            "phantom",
            "nightmare",
            "zombie",
        ];

        let pattern_lower = pattern.to_lowercase();
        tool_signatures
            .iter()
            .any(|sig| pattern_lower.contains(sig))
    }

    /// Add a known bad pattern to the detection list
    pub fn add_bad_pattern(&mut self, pattern: String) {
        self.known_bad_patterns.insert(pattern);
    }
}

impl Default for ThreatHunter {
    fn default() -> Self {
        Self::new()
    }
}
