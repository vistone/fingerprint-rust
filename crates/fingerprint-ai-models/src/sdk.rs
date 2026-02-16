//! SDK detection from User-Agent and other client identifiers

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Known SDK patterns (for future use with regex matching)
#[allow(dead_code)]
static SDK_PATTERNS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // OpenAI SDKs
    m.insert("openai-python", r"openai-python/(\d+\.\d+\.\d+)");
    m.insert("openai-node", r"openai-node/(\d+\.\d+\.\d+)");
    m.insert("openai-java", r"openai-java/(\d+\.\d+\.\d+)");
    m.insert("openai-dotnet", r"openai-dotnet/(\d+\.\d+\.\d+)");
    
    // Anthropic SDKs
    m.insert("anthropic-sdk-python", r"anthropic-sdk-python/(\d+\.\d+\.\d+)");
    m.insert("anthropic-sdk-typescript", r"@anthropic-ai/sdk/(\d+\.\d+\.\d+)");
    
    // Google SDKs
    m.insert("google-cloud-aiplatform", r"google-cloud-aiplatform/(\d+\.\d+\.\d+)");
    m.insert("google-generativeai", r"google-generativeai/(\d+\.\d+\.\d+)");
    
    // Generic patterns
    m.insert("langchain", r"langchain/(\d+\.\d+\.\d+)");
    m.insert("llamaindex", r"llama-index/(\d+\.\d+\.\d+)");
    
    m
});

/// Detect SDK from User-Agent header
pub fn detect_sdk_from_user_agent(user_agent: &str) -> Option<(String, Option<String>)> {
    let ua_lower = user_agent.to_lowercase();

    // Check for LangChain first (may contain other SDK names)
    if ua_lower.contains("langchain") {
        let version = extract_version(&ua_lower, "langchain/");
        return Some(("langchain".to_string(), version));
    }

    // Check for LlamaIndex (may contain other SDK names)
    if ua_lower.contains("llama") && (ua_lower.contains("index") || ua_lower.contains("llamaindex")) {
        let version = extract_version(&ua_lower, "llama-index/")
            .or_else(|| extract_version(&ua_lower, "llamaindex/"));
        return Some(("llama-index".to_string(), version));
    }

    // Check for OpenAI Python SDK
    if ua_lower.contains("openai-python") {
        let version = extract_version(&ua_lower, "openai-python/");
        return Some(("openai-python".to_string(), version));
    }

    // Check for OpenAI Node.js SDK
    if ua_lower.contains("openai-node") || ua_lower.contains("openai/node") {
        let version = extract_version(&ua_lower, "openai-node/")
            .or_else(|| extract_version(&ua_lower, "openai/"));
        return Some(("openai-node".to_string(), version));
    }

    // Check for Anthropic SDK
    if ua_lower.contains("anthropic") && (ua_lower.contains("sdk") || ua_lower.contains("python")) {
        let version = extract_version(&ua_lower, "anthropic-sdk-python/")
            .or_else(|| extract_version(&ua_lower, "anthropic-python/"));
        return Some(("anthropic-sdk-python".to_string(), version));
    }

    // Check for Anthropic TypeScript SDK
    if ua_lower.contains("@anthropic-ai/sdk") {
        let version = extract_version(&ua_lower, "@anthropic-ai/sdk/");
        return Some(("anthropic-sdk-typescript".to_string(), version));
    }

    // Check for Google Cloud SDKs
    if ua_lower.contains("google-cloud-aiplatform") {
        let version = extract_version(&ua_lower, "google-cloud-aiplatform/");
        return Some(("google-cloud-aiplatform".to_string(), version));
    }

    if ua_lower.contains("google-generativeai") {
        let version = extract_version(&ua_lower, "google-generativeai/");
        return Some(("google-generativeai".to_string(), version));
    }

    // Check for Python (generic)
    if ua_lower.contains("python/") && ua_lower.contains("aiohttp") {
        return Some(("python-aiohttp".to_string(), None));
    }

    if ua_lower.contains("python-requests/") {
        let version = extract_version(&ua_lower, "python-requests/");
        return Some(("python-requests".to_string(), version));
    }

    // Check for Node.js (generic)
    if ua_lower.contains("node.js") || ua_lower.contains("node/") {
        let version = extract_version(&ua_lower, "node/");
        return Some(("nodejs".to_string(), version));
    }

    // Check for cURL
    if ua_lower.starts_with("curl/") {
        let version = extract_version(&ua_lower, "curl/");
        return Some(("curl".to_string(), version));
    }

    None
}

/// Extract version number from a string after a prefix
fn extract_version(text: &str, prefix: &str) -> Option<String> {
    if let Some(start_idx) = text.find(prefix) {
        let version_start = start_idx + prefix.len();
        let remaining = &text[version_start..];
        
        // Extract version number (digits and dots)
        let version: String = remaining
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        
        if !version.is_empty() {
            return Some(version);
        }
    }
    
    None
}

/// Analyze SDK version for security vulnerabilities
pub fn check_sdk_version_security(sdk: &str, version: &str) -> SecurityLevel {
    // This is a simplified example - in practice, you'd query a vulnerability database
    
    match sdk {
        "openai-python" => {
            // Example: versions < 1.0.0 had security issues
            if version_less_than(version, "1.0.0") {
                SecurityLevel::Vulnerable
            } else if version_less_than(version, "1.10.0") {
                SecurityLevel::Outdated
            } else {
                SecurityLevel::Current
            }
        }
        "anthropic-sdk-python" => {
            if version_less_than(version, "0.5.0") {
                SecurityLevel::Outdated
            } else {
                SecurityLevel::Current
            }
        }
        _ => SecurityLevel::Unknown,
    }
}

/// Security level classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Known vulnerable version
    Vulnerable,
    /// Outdated but not known vulnerable
    Outdated,
    /// Current version
    Current,
    /// Unknown SDK or version
    Unknown,
}

/// Simple version comparison (major.minor.patch)
fn version_less_than(version: &str, threshold: &str) -> bool {
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() >= 3 {
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            ))
        } else if parts.len() == 2 {
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                0,
            ))
        } else {
            None
        }
    };

    if let (Some(v), Some(t)) = (parse_version(version), parse_version(threshold)) {
        v < t
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_openai_python_sdk() {
        let ua = "openai-python/1.3.5 Python/3.9";
        let result = detect_sdk_from_user_agent(ua);
        
        assert!(result.is_some());
        let (sdk, version) = result.unwrap();
        assert_eq!(sdk, "openai-python");
        assert_eq!(version, Some("1.3.5".to_string()));
    }

    #[test]
    fn test_detect_openai_node_sdk() {
        let ua = "openai-node/4.20.0 node/18.0.0";
        let result = detect_sdk_from_user_agent(ua);
        
        assert!(result.is_some());
        let (sdk, version) = result.unwrap();
        assert_eq!(sdk, "openai-node");
        assert_eq!(version, Some("4.20.0".to_string()));
    }

    #[test]
    fn test_detect_anthropic_sdk() {
        let ua = "anthropic-sdk-python/0.7.0 Python/3.11";
        let result = detect_sdk_from_user_agent(ua);
        
        assert!(result.is_some());
        let (sdk, _) = result.unwrap();
        assert_eq!(sdk, "anthropic-sdk-python");
    }

    #[test]
    fn test_detect_langchain() {
        let ua = "langchain/0.1.0 openai-python/1.0.0";
        let result = detect_sdk_from_user_agent(ua);
        
        assert!(result.is_some());
        let (sdk, _) = result.unwrap();
        assert_eq!(sdk, "langchain");
    }

    #[test]
    fn test_extract_version() {
        assert_eq!(extract_version("openai-python/1.2.3 other", "openai-python/"), Some("1.2.3".to_string()));
        assert_eq!(extract_version("node/18.0.0", "node/"), Some("18.0.0".to_string()));
        assert_eq!(extract_version("no-version-here", "test/"), None);
    }

    #[test]
    fn test_version_comparison() {
        assert!(version_less_than("1.0.0", "1.1.0"));
        assert!(version_less_than("1.0.0", "2.0.0"));
        assert!(!version_less_than("1.5.0", "1.4.0"));
        assert!(!version_less_than("2.0.0", "1.9.9"));
    }

    #[test]
    fn test_security_level_check() {
        assert_eq!(
            check_sdk_version_security("openai-python", "0.28.0"),
            SecurityLevel::Vulnerable
        );
        assert_eq!(
            check_sdk_version_security("openai-python", "1.5.0"),
            SecurityLevel::Outdated
        );
        assert_eq!(
            check_sdk_version_security("openai-python", "1.20.0"),
            SecurityLevel::Current
        );
    }
}
