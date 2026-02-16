//! HTTP header-based AI provider detection

use crate::{AiProvider, AiProviderFingerprint};
use std::collections::HashMap;

/// Detect AI provider from HTTP headers
pub fn detect_from_headers(headers: &HashMap<String, String>) -> Option<AiProviderFingerprint> {
    // Check for OpenAI-specific headers
    if headers.contains_key("openai-organization") || headers.contains_key("openai-version") {
        return Some(detect_openai(headers));
    }

    // Check for Anthropic-specific headers
    if headers.contains_key("anthropic-version")
        || headers
            .get("x-api-key")
            .map(|v| v.starts_with("sk-ant-"))
            .unwrap_or(false)
    {
        return Some(detect_anthropic(headers));
    }

    // Check for Google-specific headers
    if headers.contains_key("x-goog-api-client") || headers.contains_key("x-goog-user-project") {
        return Some(detect_google(headers));
    }

    // Check for Azure OpenAI
    if headers.contains_key("api-key") && headers.contains_key("api-version") {
        return Some(detect_azure_openai(headers));
    }

    // Check for Cohere
    if headers.contains_key("cohere-version") {
        return Some(detect_cohere(headers));
    }

    // Check for AWS Bedrock
    if headers.keys().any(|k| k.starts_with("x-amzn-bedrock-")) {
        return Some(detect_aws_bedrock(headers));
    }

    // Check authorization header patterns
    if let Some(auth) = headers.get("authorization") {
        if auth.starts_with("Bearer sk-") {
            // Could be OpenAI or Mistral
            return Some(
                AiProviderFingerprint::new(AiProvider::OpenAI, 0.7).with_auth("Bearer".to_string()),
            );
        } else if auth.starts_with("Bearer ey") {
            // Likely JWT token - could be Google or others
            return Some(
                AiProviderFingerprint::new(AiProvider::Other("jwt_auth".to_string()), 0.5)
                    .with_auth("Bearer JWT".to_string()),
            );
        }
    }

    None
}

/// Detect OpenAI from headers
fn detect_openai(headers: &HashMap<String, String>) -> AiProviderFingerprint {
    let mut fp =
        AiProviderFingerprint::new(AiProvider::OpenAI, 0.95).with_auth("Bearer".to_string());

    if let Some(org) = headers.get("openai-organization") {
        fp = fp.with_metadata("organization".to_string(), org.clone());
    }

    if let Some(version) = headers.get("openai-version") {
        fp = fp.with_metadata("api_version".to_string(), version.clone());
    }

    if let Some(beta) = headers.get("openai-beta") {
        fp = fp.with_metadata("beta_features".to_string(), beta.clone());
    }

    // Try to extract SDK info from user-agent
    if let Some(ua) = headers.get("user-agent") {
        if let Some((sdk, version)) = super::sdk::detect_sdk_from_user_agent(ua) {
            fp = fp.with_sdk(sdk, version);
        }
    }

    fp
}

/// Detect Anthropic Claude from headers
fn detect_anthropic(headers: &HashMap<String, String>) -> AiProviderFingerprint {
    let mut fp =
        AiProviderFingerprint::new(AiProvider::Anthropic, 0.95).with_auth("x-api-key".to_string());

    if let Some(version) = headers.get("anthropic-version") {
        fp = fp.with_metadata("api_version".to_string(), version.clone());
    }

    if let Some(beta) = headers.get("anthropic-beta") {
        fp = fp.with_metadata("beta_features".to_string(), beta.clone());
    }

    // Try to extract SDK info
    if let Some(ua) = headers.get("user-agent") {
        if let Some((sdk, version)) = super::sdk::detect_sdk_from_user_agent(ua) {
            fp = fp.with_sdk(sdk, version);
        }
    }

    fp
}

/// Detect Google Gemini from headers
fn detect_google(headers: &HashMap<String, String>) -> AiProviderFingerprint {
    let mut fp = AiProviderFingerprint::new(AiProvider::GoogleGemini, 0.9)
        .with_auth("OAuth 2.0".to_string());

    if let Some(client) = headers.get("x-goog-api-client") {
        fp = fp.with_metadata("api_client".to_string(), client.clone());
    }

    if let Some(project) = headers.get("x-goog-user-project") {
        fp = fp.with_metadata("project".to_string(), project.clone());
    }

    fp
}

/// Detect Azure OpenAI from headers
fn detect_azure_openai(headers: &HashMap<String, String>) -> AiProviderFingerprint {
    let mut fp =
        AiProviderFingerprint::new(AiProvider::AzureOpenAI, 0.9).with_auth("api-key".to_string());

    if let Some(version) = headers.get("api-version") {
        fp = fp.with_metadata("api_version".to_string(), version.clone());
    }

    fp
}

/// Detect Cohere from headers
fn detect_cohere(headers: &HashMap<String, String>) -> AiProviderFingerprint {
    let mut fp =
        AiProviderFingerprint::new(AiProvider::Cohere, 0.95).with_auth("Bearer".to_string());

    if let Some(version) = headers.get("cohere-version") {
        fp = fp.with_metadata("api_version".to_string(), version.clone());
    }

    fp
}

/// Detect AWS Bedrock from headers
fn detect_aws_bedrock(headers: &HashMap<String, String>) -> AiProviderFingerprint {
    let mut fp = AiProviderFingerprint::new(AiProvider::AwsBedrock, 0.9)
        .with_auth("AWS Signature V4".to_string());

    // Extract Bedrock-specific headers
    for (key, value) in headers {
        if key.starts_with("x-amzn-bedrock-") {
            fp = fp.with_metadata(key.clone(), value.clone());
        }
    }

    if let Some(target) = headers.get("x-amz-target") {
        fp = fp.with_metadata("amz_target".to_string(), target.clone());
    }

    fp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_openai_headers() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer sk-...".to_string());
        headers.insert("openai-organization".to_string(), "org-123".to_string());
        headers.insert("openai-version".to_string(), "2023-05-15".to_string());

        let result = detect_from_headers(&headers);
        assert!(result.is_some());

        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::OpenAI);
        assert!(fp.confidence > 0.9);
        assert!(fp.metadata.contains_key("organization"));
    }

    #[test]
    fn test_detect_anthropic_headers() {
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), "sk-ant-...".to_string());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());

        let result = detect_from_headers(&headers);
        assert!(result.is_some());

        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::Anthropic);
        assert!(fp.confidence > 0.9);
    }

    #[test]
    fn test_detect_google_headers() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer eyJ...".to_string());
        headers.insert("x-goog-api-client".to_string(), "gl-go/1.0".to_string());

        let result = detect_from_headers(&headers);
        assert!(result.is_some());

        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::GoogleGemini);
    }

    #[test]
    fn test_detect_azure_openai_headers() {
        let mut headers = HashMap::new();
        headers.insert("api-key".to_string(), "abc123".to_string());
        headers.insert("api-version".to_string(), "2023-05-15".to_string());

        let result = detect_from_headers(&headers);
        assert!(result.is_some());

        let fp = result.unwrap();
        assert_eq!(fp.provider, AiProvider::AzureOpenAI);
    }
}
