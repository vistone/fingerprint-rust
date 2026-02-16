//! AI Provider definitions and characteristics

use crate::AiProvider;

/// Provider characteristics for matching
#[derive(Debug, Clone)]
pub struct ProviderCharacteristics {
    /// Provider identifier
    pub provider: AiProvider,
    
    /// Known API endpoints
    pub endpoints: Vec<&'static str>,
    
    /// Known authentication header keys
    pub auth_headers: Vec<&'static str>,
    
    /// Custom headers specific to this provider
    pub custom_headers: Vec<&'static str>,
    
    /// Known model name patterns
    pub model_patterns: Vec<&'static str>,
    
    /// Known domains
    pub domains: Vec<&'static str>,
}

impl ProviderCharacteristics {
    /// Get all provider characteristics
    pub fn all_providers() -> Vec<Self> {
        vec![
            Self::openai(),
            Self::anthropic(),
            Self::google_gemini(),
            Self::azure_openai(),
            Self::mistral(),
            Self::cohere(),
            Self::huggingface(),
            Self::aws_bedrock(),
        ]
    }

    /// OpenAI characteristics
    pub fn openai() -> Self {
        Self {
            provider: AiProvider::OpenAI,
            endpoints: vec![
                "/v1/chat/completions",
                "/v1/completions",
                "/v1/edits",
                "/v1/embeddings",
                "/v1/models",
                "/v1/images/generations",
                "/v1/audio/transcriptions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["openai-organization", "openai-version", "openai-beta"],
            model_patterns: vec!["gpt-4", "gpt-3.5", "gpt-4-turbo", "text-davinci", "text-embedding"],
            domains: vec!["api.openai.com", "openai.com"],
        }
    }

    /// Anthropic Claude characteristics
    pub fn anthropic() -> Self {
        Self {
            provider: AiProvider::Anthropic,
            endpoints: vec![
                "/v1/messages",
                "/v1/complete",
                "/v1/models",
            ],
            auth_headers: vec!["x-api-key"],
            custom_headers: vec!["anthropic-version", "anthropic-beta"],
            model_patterns: vec!["claude-3", "claude-2", "claude-instant"],
            domains: vec!["api.anthropic.com", "anthropic.com"],
        }
    }

    /// Google Gemini characteristics
    pub fn google_gemini() -> Self {
        Self {
            provider: AiProvider::GoogleGemini,
            endpoints: vec![
                "/v1/projects",
                "/v1beta/models",
                "publishers/google/models",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["x-goog-api-client", "x-goog-user-project"],
            model_patterns: vec!["gemini-pro", "gemini-ultra", "gemini-1.5"],
            domains: vec!["aiplatform.googleapis.com", "generativelanguage.googleapis.com"],
        }
    }

    /// Azure OpenAI characteristics
    pub fn azure_openai() -> Self {
        Self {
            provider: AiProvider::AzureOpenAI,
            endpoints: vec![
                "/openai/deployments",
                "/openai/models",
            ],
            auth_headers: vec!["api-key", "authorization"],
            custom_headers: vec!["api-version"],
            model_patterns: vec!["gpt-4", "gpt-35-turbo"],
            domains: vec!["openai.azure.com", "cognitiveservices.azure.com"],
        }
    }

    /// Mistral AI characteristics
    pub fn mistral() -> Self {
        Self {
            provider: AiProvider::Mistral,
            endpoints: vec![
                "/v1/chat/completions",
                "/v1/embeddings",
                "/v1/models",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["mistral-tiny", "mistral-small", "mistral-medium", "mistral-large"],
            domains: vec!["api.mistral.ai"],
        }
    }

    /// Cohere characteristics
    pub fn cohere() -> Self {
        Self {
            provider: AiProvider::Cohere,
            endpoints: vec![
                "/v1/generate",
                "/v1/embed",
                "/v1/classify",
                "/v1/chat",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["cohere-version"],
            model_patterns: vec!["command", "command-r", "embed-english"],
            domains: vec!["api.cohere.ai", "cohere.ai"],
        }
    }

    /// Hugging Face characteristics
    pub fn huggingface() -> Self {
        Self {
            provider: AiProvider::HuggingFace,
            endpoints: vec![
                "/models",
                "/pipeline",
                "/v1/chat/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["meta-llama", "mistralai", "google", "microsoft"],
            domains: vec!["api-inference.huggingface.co", "huggingface.co"],
        }
    }

    /// AWS Bedrock characteristics
    pub fn aws_bedrock() -> Self {
        Self {
            provider: AiProvider::AwsBedrock,
            endpoints: vec![
                "/model",
                "/bedrock-runtime",
            ],
            auth_headers: vec!["authorization", "x-amz-date"],
            custom_headers: vec!["x-amzn-bedrock-", "x-amz-target"],
            model_patterns: vec!["anthropic.claude", "amazon.titan", "ai21.j2"],
            domains: vec!["bedrock-runtime.amazonaws.com", "bedrock.amazonaws.com"],
        }
    }

    /// Check if endpoint matches this provider
    pub fn matches_endpoint(&self, endpoint: &str) -> bool {
        self.endpoints.iter().any(|pattern| endpoint.contains(pattern))
    }

    /// Check if domain matches this provider
    pub fn matches_domain(&self, domain: &str) -> bool {
        self.domains.iter().any(|d| domain.contains(d))
    }

    /// Check if model name matches this provider
    pub fn matches_model(&self, model: &str) -> bool {
        self.model_patterns.iter().any(|pattern| model.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_characteristics() {
        let openai = ProviderCharacteristics::openai();
        assert_eq!(openai.provider, AiProvider::OpenAI);
        assert!(openai.matches_endpoint("/v1/chat/completions"));
        assert!(openai.matches_model("gpt-4"));
        assert!(openai.matches_domain("api.openai.com"));
    }

    #[test]
    fn test_anthropic_characteristics() {
        let anthropic = ProviderCharacteristics::anthropic();
        assert_eq!(anthropic.provider, AiProvider::Anthropic);
        assert!(anthropic.matches_endpoint("/v1/messages"));
        assert!(anthropic.matches_model("claude-3-opus"));
    }

    #[test]
    fn test_all_providers() {
        let providers = ProviderCharacteristics::all_providers();
        assert!(providers.len() >= 8);
    }
}
