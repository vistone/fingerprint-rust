//! API endpoint and request body pattern detection

use crate::{AiProvider, AiProviderFingerprint, providers::ProviderCharacteristics};
use std::collections::HashMap;

/// Detect AI provider from API endpoint pattern
pub fn detect_from_endpoint(endpoint: &str, headers: &HashMap<String, String>) -> Option<AiProviderFingerprint> {
    let providers = ProviderCharacteristics::all_providers();

    for provider_chars in providers {
        if provider_chars.matches_endpoint(endpoint) {
            let mut confidence = 0.7;

            // Increase confidence if headers also match
            let has_auth_header = provider_chars.auth_headers.iter()
                .any(|h| headers.contains_key(*h));
            
            let has_custom_header = provider_chars.custom_headers.iter()
                .any(|h| headers.contains_key(*h));

            if has_auth_header {
                confidence += 0.1;
            }
            if has_custom_header {
                confidence += 0.15;
            }

            let mut fp = AiProviderFingerprint::new(provider_chars.provider.clone(), confidence)
                .with_endpoint(endpoint.to_string());

            // Try to extract model name from endpoint
            if let Some(model) = extract_model_from_endpoint(endpoint) {
                fp = fp.with_model(model);
            }

            return Some(fp);
        }
    }

    None
}

/// Detect AI provider from request body
pub fn detect_from_body(body: &str, _headers: &HashMap<String, String>) -> Option<AiProviderFingerprint> {
    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // Check for model field
        if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
            let providers = ProviderCharacteristics::all_providers();
            
            for provider_chars in providers {
                if provider_chars.matches_model(model) {
                    return Some(
                        AiProviderFingerprint::new(provider_chars.provider.clone(), 0.8)
                            .with_model(model.to_string())
                    );
                }
            }
        }

        // Check for Anthropic-specific fields
        if json.get("max_tokens").is_some() && json.get("system").is_some() {
            return Some(AiProviderFingerprint::new(AiProvider::Anthropic, 0.6));
        }

        // Check for OpenAI-specific fields
        if json.get("messages").is_some() && json.get("temperature").is_some() {
            return Some(AiProviderFingerprint::new(AiProvider::OpenAI, 0.5));
        }
    }

    None
}

/// Extract model name from endpoint path
fn extract_model_from_endpoint(endpoint: &str) -> Option<String> {
    // Azure OpenAI pattern: /openai/deployments/{deployment-id}/...
    if endpoint.contains("/openai/deployments/") {
        let parts: Vec<&str> = endpoint.split('/').collect();
        if let Some(idx) = parts.iter().position(|&p| p == "deployments") {
            if idx + 1 < parts.len() {
                return Some(parts[idx + 1].to_string());
            }
        }
    }

    // Google Vertex AI pattern: .../models/{model-name}
    if endpoint.contains("/models/") {
        let parts: Vec<&str> = endpoint.split('/').collect();
        if let Some(idx) = parts.iter().position(|&p| p == "models") {
            if idx + 1 < parts.len() {
                return Some(parts[idx + 1].to_string());
            }
        }
    }

    None
}

/// Analyze request patterns for bot detection
pub fn analyze_request_pattern(
    endpoints: &[(&str, u64)],  // (endpoint, timestamp)
) -> f32 {
    if endpoints.is_empty() {
        return 0.0;
    }

    // Calculate request frequency
    let mut intervals = Vec::new();
    for i in 1..endpoints.len() {
        let interval = endpoints[i].1 - endpoints[i - 1].1;
        intervals.push(interval);
    }

    if intervals.is_empty() {
        return 0.0;
    }

    // Calculate variance in intervals
    let mean = intervals.iter().sum::<u64>() as f32 / intervals.len() as f32;
    let variance: f32 = intervals.iter()
        .map(|&i| {
            let diff = i as f32 - mean;
            diff * diff
        })
        .sum::<f32>() / intervals.len() as f32;

    // Low variance = more bot-like
    // High variance = more human-like
    let normalized_variance = (variance / (mean * mean)).min(1.0);
    
    // Return human-likeness score (0.0 = bot, 1.0 = human)
    normalized_variance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_openai_endpoint() {
        let headers = HashMap::new();
        let result = detect_from_endpoint("/v1/chat/completions", &headers);
        
        assert!(result.is_some());
        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::OpenAI);
        assert!(fp.endpoint.is_some());
    }

    #[test]
    fn test_detect_from_anthropic_endpoint() {
        let headers = HashMap::new();
        let result = detect_from_endpoint("/v1/messages", &headers);
        
        assert!(result.is_some());
        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::Anthropic);
    }

    #[test]
    fn test_detect_from_body_with_model() {
        let body = r#"{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}"#;
        let headers = HashMap::new();
        
        let result = detect_from_body(body, &headers);
        assert!(result.is_some());
        
        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::OpenAI);
        assert_eq!(fp.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_detect_from_body_claude() {
        let body = r#"{"model": "claude-3-opus", "max_tokens": 1024}"#;
        let headers = HashMap::new();
        
        let result = detect_from_body(body, &headers);
        assert!(result.is_some());
        
        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::Anthropic);
    }

    #[test]
    fn test_extract_model_from_azure_endpoint() {
        let endpoint = "/openai/deployments/gpt-4-deployment/chat/completions";
        let model = extract_model_from_endpoint(endpoint);
        
        assert_eq!(model, Some("gpt-4-deployment".to_string()));
    }

    #[test]
    fn test_analyze_regular_pattern() {
        // Regular intervals (bot-like)
        let endpoints = vec![
            ("/v1/chat", 1000),
            ("/v1/chat", 2000),
            ("/v1/chat", 3000),
            ("/v1/chat", 4000),
        ];
        
        let score = analyze_request_pattern(&endpoints);
        assert!(score < 0.1); // Very bot-like
    }

    #[test]
    fn test_analyze_human_pattern() {
        // Irregular intervals (human-like)
        let endpoints = vec![
            ("/v1/chat", 1000),
            ("/v1/chat", 1500),
            ("/v1/chat", 3200),
            ("/v1/chat", 8000),
        ];
        
        let score = analyze_request_pattern(&endpoints);
        assert!(score > 0.3); // More human-like
    }
}
