//! # fingerprint-ai-models
//!
//! **AI Model Fingerprinting for Generative Models**
//!
//! This module provides comprehensive fingerprinting capabilities for detecting and identifying
//! requests to various AI model providers including OpenAI, Anthropic Claude, Google Gemini,
//! and other generative AI services.
//!
//! ## Features
//!
//! - **Provider Detection**: Identify OpenAI, Anthropic, Google, Azure OpenAI, and other AI providers
//! - **HTTP Header Analysis**: Detect provider-specific authentication and versioning headers
//! - **TLS Fingerprinting**: Match JA3 patterns for different AI service infrastructure
//! - **SDK Detection**: Identify client libraries (Python, Node.js, JavaScript, etc.)
//! - **API Pattern Analysis**: Recognize endpoint structures and request patterns
//! - **Model Version Detection**: Identify specific AI models in use (GPT-4, Claude 3, etc.)
//!
//! ## Example
//!
//! ```rust
//! use fingerprint_ai_models::{detect_ai_provider, AiProviderFingerprint};
//!
//! let headers = vec![
//!     ("Authorization".to_string(), "Bearer sk-...".to_string()),
//!     ("OpenAI-Organization".to_string(), "org-...".to_string()),
//! ];
//!
//! let result = detect_ai_provider(&headers, "/v1/chat/completions", None);
//! assert!(result.is_some());
//! ```

pub mod providers;
pub mod headers;
pub mod patterns;
pub mod sdk;
pub mod tls;
pub mod content_detection;
pub mod audio_detection;
pub mod video_detection;
pub mod image_detection;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI Model Provider Types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AiProvider {
    // US/Western Providers
    /// OpenAI (GPT-3.5, GPT-4, etc.)
    OpenAI,
    /// Anthropic Claude (Claude 3, Claude 3.5, etc.)
    Anthropic,
    /// Google Gemini (via Vertex AI)
    GoogleGemini,
    /// Azure OpenAI Service
    AzureOpenAI,
    /// Mistral AI
    Mistral,
    /// Cohere
    Cohere,
    /// Meta Llama (via various providers)
    MetaLlama,
    /// Hugging Face Inference API
    HuggingFace,
    /// AWS Bedrock
    AwsBedrock,
    /// Perplexity AI
    PerplexityAI,
    /// xAI Grok
    XAI,
    /// Replicate
    Replicate,
    /// Stability AI
    StabilityAI,
    /// AI21 Labs
    AI21Labs,
    
    // Chinese Providers
    /// Alibaba Cloud - Qwen/Tongyi Qianwen (通义千问)
    AlibabaQwen,
    /// Baidu - ERNIE Bot/Wenxin Yiyan (文心一言)
    BaiduErnie,
    /// Tencent - Hunyuan (混元)
    TencentHunyuan,
    /// ByteDance - Doubao/Coze (豆包)
    ByteDanceDoubao,
    /// Zhipu AI - ChatGLM/GLM-4 (智谱)
    ZhipuGLM,
    /// Moonshot AI - Kimi (月之暗面)
    MoonshotKimi,
    /// DeepSeek
    DeepSeek,
    /// MiniMax - Abab
    MiniMax,
    /// SenseTime - SenseChat/SenseNova (商汤)
    SenseTime,
    /// iFlytek - Spark/Xinghuo (科大讯飞/星火)
    IFlytekSpark,
    /// 01.AI - Yi (零一万物)
    ZeroOneAI,
    /// Baichuan (百川)
    Baichuan,
    
    // Other Global Providers
    /// Reka AI (Singapore)
    RekaAI,
    
    /// Other/Unknown provider
    Other(String),
}

impl AiProvider {
    /// Get provider name as string
    pub fn as_str(&self) -> &str {
        match self {
            // US/Western
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::GoogleGemini => "google_gemini",
            Self::AzureOpenAI => "azure_openai",
            Self::Mistral => "mistral",
            Self::Cohere => "cohere",
            Self::MetaLlama => "meta_llama",
            Self::HuggingFace => "huggingface",
            Self::AwsBedrock => "aws_bedrock",
            Self::PerplexityAI => "perplexity",
            Self::XAI => "xai",
            Self::Replicate => "replicate",
            Self::StabilityAI => "stability_ai",
            Self::AI21Labs => "ai21",
            // Chinese
            Self::AlibabaQwen => "alibaba_qwen",
            Self::BaiduErnie => "baidu_ernie",
            Self::TencentHunyuan => "tencent_hunyuan",
            Self::ByteDanceDoubao => "bytedance_doubao",
            Self::ZhipuGLM => "zhipu_glm",
            Self::MoonshotKimi => "moonshot_kimi",
            Self::DeepSeek => "deepseek",
            Self::MiniMax => "minimax",
            Self::SenseTime => "sensetime",
            Self::IFlytekSpark => "iflytek_spark",
            Self::ZeroOneAI => "01ai",
            Self::Baichuan => "baichuan",
            // Other Global
            Self::RekaAI => "reka",
            Self::Other(name) => name,
        }
    }
}

/// AI Model Fingerprint Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderFingerprint {
    /// Detected provider
    pub provider: AiProvider,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    
    /// Detected model name (e.g., "gpt-4", "claude-3-opus")
    pub model: Option<String>,
    
    /// SDK type if detected
    pub sdk: Option<String>,
    
    /// SDK version if detected
    pub sdk_version: Option<String>,
    
    /// Authentication method detected
    pub auth_method: Option<String>,
    
    /// API endpoint detected
    pub endpoint: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AiProviderFingerprint {
    /// Create a new fingerprint result
    pub fn new(provider: AiProvider, confidence: f32) -> Self {
        Self {
            provider,
            confidence,
            model: None,
            sdk: None,
            sdk_version: None,
            auth_method: None,
            endpoint: None,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set model name
    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    /// Set SDK information
    pub fn with_sdk(mut self, sdk: String, version: Option<String>) -> Self {
        self.sdk = Some(sdk);
        self.sdk_version = version;
        self
    }

    /// Set authentication method
    pub fn with_auth(mut self, auth_method: String) -> Self {
        self.auth_method = Some(auth_method);
        self
    }

    /// Set endpoint
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

/// Detect AI provider from HTTP headers and endpoint
///
/// # Arguments
///
/// * `headers` - HTTP headers as key-value pairs
/// * `endpoint` - Request endpoint/path
/// * `body` - Optional request body for additional analysis
///
/// # Returns
///
/// `Some(AiProviderFingerprint)` if a provider is detected, `None` otherwise
///
/// # Example
///
/// ```rust
/// use fingerprint_ai_models::detect_ai_provider;
///
/// let headers = vec![
///     ("Authorization".to_string(), "Bearer sk-...".to_string()),
/// ];
/// let result = detect_ai_provider(&headers, "/v1/chat/completions", None);
/// ```
pub fn detect_ai_provider(
    headers: &[(String, String)],
    endpoint: &str,
    body: Option<&str>,
) -> Option<AiProviderFingerprint> {
    // Convert headers to HashMap for easier lookup
    let header_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v.clone()))
        .collect();

    // Check headers for provider-specific patterns
    if let Some(fp) = headers::detect_from_headers(&header_map) {
        return Some(fp);
    }

    // Check endpoint patterns
    if let Some(fp) = patterns::detect_from_endpoint(endpoint, &header_map) {
        return Some(fp);
    }

    // Check request body if available
    if let Some(body_str) = body {
        if let Some(fp) = patterns::detect_from_body(body_str, &header_map) {
            return Some(fp);
        }
    }

    None
}

/// Detect SDK from User-Agent header
///
/// # Arguments
///
/// * `user_agent` - User-Agent header value
///
/// # Returns
///
/// `Some((sdk_name, sdk_version))` if detected
pub fn detect_sdk(user_agent: &str) -> Option<(String, Option<String>)> {
    sdk::detect_sdk_from_user_agent(user_agent)
}

/// Get provider fingerprint from TLS/JA3 hash
///
/// # Arguments
///
/// * `ja3_hash` - JA3 hash of TLS connection
///
/// # Returns
///
/// `Some(AiProvider)` if recognized pattern found
pub fn detect_from_ja3(ja3_hash: &str) -> Option<AiProvider> {
    tls::detect_provider_from_ja3(ja3_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_provider_as_str() {
        assert_eq!(AiProvider::OpenAI.as_str(), "openai");
        assert_eq!(AiProvider::Anthropic.as_str(), "anthropic");
        assert_eq!(AiProvider::GoogleGemini.as_str(), "google_gemini");
    }

    #[test]
    fn test_fingerprint_builder() {
        let fp = AiProviderFingerprint::new(AiProvider::OpenAI, 0.95)
            .with_model("gpt-4".to_string())
            .with_sdk("openai-python".to_string(), Some("1.0.0".to_string()))
            .with_auth("Bearer".to_string())
            .with_endpoint("/v1/chat/completions".to_string())
            .with_metadata("region".to_string(), "us-east-1".to_string());

        assert_eq!(fp.provider, AiProvider::OpenAI);
        assert_eq!(fp.confidence, 0.95);
        assert_eq!(fp.model, Some("gpt-4".to_string()));
        assert!(fp.metadata.contains_key("region"));
    }

    #[test]
    fn test_detect_openai() {
        let headers = vec![
            ("Authorization".to_string(), "Bearer sk-...".to_string()),
            ("OpenAI-Organization".to_string(), "org-123".to_string()),
        ];

        let result = detect_ai_provider(&headers, "/v1/chat/completions", None);
        assert!(result.is_some());
        
        if let Some(fp) = result {
            assert_eq!(fp.provider, AiProvider::OpenAI);
            assert!(fp.confidence > 0.8);
        }
    }

    #[test]
    fn test_detect_anthropic() {
        let headers = vec![
            ("x-api-key".to_string(), "sk-ant-...".to_string()),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ];

        let result = detect_ai_provider(&headers, "/v1/messages", None);
        assert!(result.is_some());
        
        if let Some(fp) = result {
            assert_eq!(fp.provider, AiProvider::Anthropic);
            assert!(fp.confidence > 0.8);
        }
    }
}
