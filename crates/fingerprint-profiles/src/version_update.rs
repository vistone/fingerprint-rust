/// Browser Version Update Management Tool
///
/// Automated script for adding and updating browser version profiles
/// Reduces manual work when new browser versions are released
use std::collections::HashMap;

/// Browser version update configuration
#[derive(Debug, Clone)]
pub struct VersionUpdateConfig {
    /// New version number to add
    pub version: u32,
    /// Browser type (chrome, firefox, safari, edge, opera)
    pub browser: String,
    /// Release date in YYYY-MM-DD format
    pub release_date: String,
    /// TLS 1.3 support
    pub tls13_support: bool,
    /// ECH support
    pub ech_support: bool,
    /// HTTP/2 support
    pub http2_support: bool,
    /// HTTP/3 support
    pub http3_support: bool,
    /// PSK support
    pub psk_support: bool,
    /// 0-RTT support
    pub early_data_support: bool,
    /// Post-quantum support
    pub pq_support: bool,
    /// Fallback version for compatibility
    pub fallback_version: Option<u32>,
}

/// Version update manager
pub struct VersionUpdateManager;

impl VersionUpdateManager {
    /// Generate registry code for new version
    ///
    /// # Example
    ///
    /// ```
    /// use fingerprint_profiles::version_update::*;
    ///
    /// let config = VersionUpdateConfig {
    ///     version: 140,
    ///     browser: "chrome".to_string(),
    ///     release_date: "2025-07-01".to_string(),
    ///     tls13_support: true,
    ///     ech_support: true,
    ///     http2_support: true,
    ///     http3_support: true,
    ///     psk_support: true,
    ///     early_data_support: true,
    ///     pq_support: true,
    ///     fallback_version: Some(139),
    /// };
    ///
    /// let code = VersionUpdateManager::generate_registry_code(&config);
    /// println!("{}", code);
    /// ```
    pub fn generate_registry_code(config: &VersionUpdateConfig) -> String {
        let fn_name = format!("{}_{}", config.browser, config.version);

        format!(
            "        self.add_{}_version({}, \"{}\", {}, {}, {}, {}, {}, {}, {}, {}, \"{}\");",
            config.browser,
            config.version,
            config.release_date,
            config.tls13_support,
            config.ech_support,
            config.http2_support,
            config.http3_support,
            config.psk_support,
            config.early_data_support,
            config.pq_support,
            config
                .fallback_version
                .map(|v| format!("Some({})", v))
                .unwrap_or_else(|| "None".to_string()),
            fn_name,
        )
    }

    /// Generate profile function stub
    pub fn generate_profile_stub(browser: &str, version: u32, fallback_version: u32) -> String {
        let fn_name = format!("{}_{}", browser, version);
        let fallback_fn = format!("{}_{}", browser, fallback_version);

        format!(
            r#"/// {} v{} fingerprint configuration
pub fn {fn_name}() -> BrowserProfile {{
    // TODO: Implement {} v{} specific configuration
    // Base on: {fallback_fn}() or copy from previous stable version
    {fallback_fn}()  // Temporary fallback
}}"#,
            match browser {
                "chrome" => "Chrome",
                "firefox" => "Firefox",
                "safari" => "Safari",
                "edge" => "Edge",
                "opera" => "Opera",
                _ => browser,
            },
            version,
            match browser {
                "chrome" => "Chrome",
                "firefox" => "Firefox",
                "safari" => "Safari",
                "edge" => "Edge",
                "opera" => "Opera",
                _ => browser,
            },
            version,
        )
    }

    /// Generate profile map entry
    pub fn generate_profile_map_entry(browser: &str, version: u32) -> String {
        let key = match browser {
            "chrome" => format!("chrome_{}", version),
            "firefox" => format!("firefox_{}", version),
            "safari" => format!("safari_{}", version),
            "edge" => format!("edge_{}", version),
            "opera" => format!("opera_{}", version),
            _ => format!("{}_{}", browser, version),
        };

        let fn_name = format!("{}_{}", browser, version);
        format!("    map.insert(\"{}\".to_string(), {}());", key, fn_name)
    }

    /// Generate complete update for new version
    pub fn generate_complete_update(configs: &[VersionUpdateConfig]) -> String {
        let mut output = String::new();

        output.push_str("// === Registry Code ===\n");
        for config in configs {
            output.push_str(&Self::generate_registry_code(config));
            output.push('\n');
        }

        output.push_str("\n// === Profile Function Stubs ===\n");
        for config in configs {
            let fallback = config.fallback_version.unwrap_or(config.version - 1);
            output.push_str(&Self::generate_profile_stub(
                &config.browser,
                config.version,
                fallback,
            ));
            output.push_str("\n\n");
        }

        output.push_str("// === Profile Map Entries ===\n");
        for config in configs {
            output.push_str(&Self::generate_profile_map_entry(
                &config.browser,
                config.version,
            ));
            output.push('\n');
        }

        output
    }

    /// Generate feature comparison for versions
    pub fn generate_version_comparison(versions: &[(u32, VersionUpdateConfig)]) -> String {
        let mut output = String::from("| Version | TLS1.3 | ECH | HTTP3 | PSK | 0-RTT | PQ |\n");
        output.push_str("|---------|--------|-----|-------|-----|-------|----|\n");

        for (_, config) in versions {
            output.push_str(&format!(
                "| v{:<7} | {:<6} | {:<3} | {:<5} | {:<3} | {:<5} | {:<2} |\n",
                config.version,
                if config.tls13_support { "✓" } else { "✗" },
                if config.ech_support { "✓" } else { "✗" },
                if config.http3_support { "✓" } else { "✗" },
                if config.psk_support { "✓" } else { "✗" },
                if config.early_data_support {
                    "✓"
                } else {
                    "✗"
                },
                if config.pq_support { "✓" } else { "✗" },
            ));
        }

        output
    }

    /// Get default configuration for new version (based on latest)
    pub fn get_default_config(
        browser: &str,
        new_version: u32,
        latest_version: u32,
    ) -> VersionUpdateConfig {
        VersionUpdateConfig {
            version: new_version,
            browser: browser.to_string(),
            release_date: "2025-TBD".to_string(),
            tls13_support: true,
            ech_support: latest_version >= 124, // Chrome ECH support from 124
            http2_support: true,
            http3_support: latest_version >= 120, // Chrome HTTP/3 from 120
            psk_support: latest_version >= 120,
            early_data_support: latest_version >= 120,
            pq_support: latest_version >= 130, // Chrome Kyber768 from 130
            fallback_version: Some(latest_version),
        }
    }

    /// Check required updates for version
    pub fn check_outdated_versions() -> HashMap<String, Vec<u32>> {
        let mut outdated = HashMap::new();

        // Check which versions need PSK implementation
        let psk_needed = vec!["chrome_130", "chrome_131"];
        let mut needs_psk = Vec::new();
        for v in &psk_needed {
            if let Ok(version) = v.split('_').nth(1).unwrap().parse::<u32>() {
                needs_psk.push(version);
            }
        }
        if !needs_psk.is_empty() {
            outdated.insert("psk_variants".to_string(), needs_psk);
        }

        outdated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_registry_code() {
        let config = VersionUpdateConfig {
            version: 140,
            browser: "chrome".to_string(),
            release_date: "2025-07-01".to_string(),
            tls13_support: true,
            ech_support: true,
            http2_support: true,
            http3_support: true,
            psk_support: true,
            early_data_support: true,
            pq_support: true,
            fallback_version: Some(139),
        };

        let code = VersionUpdateManager::generate_registry_code(&config);
        assert!(code.contains("chrome_140"));
        assert!(code.contains("2025-07-01"));
    }

    #[test]
    fn test_generate_profile_stub() {
        let stub = VersionUpdateManager::generate_profile_stub("chrome", 140, 139);
        assert!(stub.contains("chrome_140"));
        assert!(stub.contains("pub fn"));
    }

    #[test]
    fn test_generate_complete_update() {
        let configs = vec![VersionUpdateConfig {
            version: 140,
            browser: "chrome".to_string(),
            release_date: "2025-07-01".to_string(),
            tls13_support: true,
            ech_support: true,
            http2_support: true,
            http3_support: true,
            psk_support: true,
            early_data_support: true,
            pq_support: true,
            fallback_version: Some(139),
        }];

        let output = VersionUpdateManager::generate_complete_update(&configs);
        assert!(output.contains("Registry Code"));
        assert!(output.contains("Profile Function Stubs"));
        assert!(output.contains("Profile Map Entries"));
    }
}
