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
            // US/Western Providers
            Self::openai(),
            Self::anthropic(),
            Self::google_gemini(),
            Self::azure_openai(),
            Self::mistral(),
            Self::cohere(),
            Self::huggingface(),
            Self::aws_bedrock(),
            Self::perplexity_ai(),
            Self::xai(),
            Self::replicate(),
            Self::stability_ai(),
            Self::ai21_labs(),
            // Chinese Providers
            Self::alibaba_qwen(),
            Self::baidu_ernie(),
            Self::tencent_hunyuan(),
            Self::bytedance_doubao(),
            Self::zhipu_glm(),
            Self::moonshot_kimi(),
            Self::deepseek(),
            Self::minimax(),
            Self::sensetime(),
            Self::iflytek_spark(),
            Self::zeroone_ai(),
            Self::baichuan(),
            // Other Global
            Self::reka_ai(),
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

    // === US/Western Providers ===

    /// Perplexity AI characteristics
    pub fn perplexity_ai() -> Self {
        Self {
            provider: AiProvider::PerplexityAI,
            endpoints: vec![
                "/chat/completions",
                "/v1/chat/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["pplx-", "sonar-", "codellama-", "llama-2-", "mistral-"],
            domains: vec!["api.perplexity.ai"],
        }
    }

    /// xAI Grok characteristics
    pub fn xai() -> Self {
        Self {
            provider: AiProvider::XAI,
            endpoints: vec![
                "/v1/chat/completions",
                "/v1/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["x-api-key"],
            model_patterns: vec!["grok-", "grok-1", "grok-2"],
            domains: vec!["api.x.ai", "x.ai"],
        }
    }

    /// Replicate characteristics
    pub fn replicate() -> Self {
        Self {
            provider: AiProvider::Replicate,
            endpoints: vec![
                "/v1/predictions",
                "/v1/models",
                "/predictions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["stability-ai", "meta/llama", "mistralai"],
            domains: vec!["api.replicate.com", "replicate.com"],
        }
    }

    /// Stability AI characteristics
    pub fn stability_ai() -> Self {
        Self {
            provider: AiProvider::StabilityAI,
            endpoints: vec![
                "/v1/generation",
                "/v1/engines",
                "/v2beta/stable-image",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["stability-client-id"],
            model_patterns: vec!["stable-diffusion", "sdxl", "sd3"],
            domains: vec!["api.stability.ai", "stability.ai"],
        }
    }

    /// AI21 Labs characteristics
    pub fn ai21_labs() -> Self {
        Self {
            provider: AiProvider::AI21Labs,
            endpoints: vec![
                "/studio/v1/chat/completions",
                "/studio/v1/complete",
                "/v1/j2-",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["j2-", "jamba-", "jurassic-"],
            domains: vec!["api.ai21.com"],
        }
    }

    // === Chinese Providers ===

    /// Alibaba Cloud Qwen/Tongyi Qianwen (通义千问) characteristics
    pub fn alibaba_qwen() -> Self {
        Self {
            provider: AiProvider::AlibabaQwen,
            endpoints: vec![
                "/v1/services/aigc/text-generation/generation",
                "/api/v1/services/aigc",
                "/compatible-mode/v1/chat/completions",
            ],
            auth_headers: vec!["authorization", "x-dashscope-api-key"],
            custom_headers: vec!["x-dashscope-", "x-ds-"],
            model_patterns: vec!["qwen", "qwen-turbo", "qwen-plus", "qwen-max", "tongyi"],
            domains: vec!["dashscope.aliyuncs.com", "aliyun.com"],
        }
    }

    /// Baidu ERNIE Bot/Wenxin Yiyan (文心一言) characteristics
    pub fn baidu_ernie() -> Self {
        Self {
            provider: AiProvider::BaiduErnie,
            endpoints: vec![
                "/rpc/2.0/ai_custom/v1/wenxinworkshop",
                "/chat/completions",
                "/chat/eb-instant",
                "/chat/ernie",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["ernie-", "ernie-bot", "ernie-3.5", "ernie-4.0", "eb-instant"],
            domains: vec!["aip.baidubce.com", "baidu.com"],
        }
    }

    /// Tencent Hunyuan (混元) characteristics
    pub fn tencent_hunyuan() -> Self {
        Self {
            provider: AiProvider::TencentHunyuan,
            endpoints: vec![
                "/v1/chat/completions",
                "/hyllm/v1/chat/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["x-tc-action", "x-tc-version"],
            model_patterns: vec!["hunyuan-", "hunyuan-lite", "hunyuan-pro", "hunyuan-turbo"],
            domains: vec!["hunyuan.tencentcloudapi.com", "hunyuan.cloud.tencent.com"],
        }
    }

    /// ByteDance Doubao/Coze (豆包) characteristics
    pub fn bytedance_doubao() -> Self {
        Self {
            provider: AiProvider::ByteDanceDoubao,
            endpoints: vec![
                "/api/v3/chat/completions",
                "/v1/chat/completions",
                "/api/v1/",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["x-tt-logid"],
            model_patterns: vec!["doubao-", "ep-", "Doubao-", "skylark"],
            domains: vec!["ark.cn-beijing.volces.com", "open.bigmodel.cn", "maas-api.ml-platform-cn-beijing.volces.com"],
        }
    }

    /// Zhipu AI ChatGLM/GLM-4 (智谱) characteristics
    pub fn zhipu_glm() -> Self {
        Self {
            provider: AiProvider::ZhipuGLM,
            endpoints: vec![
                "/api/paas/v4/chat/completions",
                "/api/paas/v3/model-api",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["glm-4", "glm-3", "chatglm", "codegeex"],
            domains: vec!["open.bigmodel.cn", "zhipuai.cn"],
        }
    }

    /// Moonshot AI Kimi (月之暗面) characteristics
    pub fn moonshot_kimi() -> Self {
        Self {
            provider: AiProvider::MoonshotKimi,
            endpoints: vec![
                "/v1/chat/completions",
                "/v1/models",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["moonshot-v1", "kimi-", "moonshot-"],
            domains: vec!["api.moonshot.cn"],
        }
    }

    /// DeepSeek characteristics
    pub fn deepseek() -> Self {
        Self {
            provider: AiProvider::DeepSeek,
            endpoints: vec![
                "/v1/chat/completions",
                "/chat/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["deepseek-chat", "deepseek-coder"],
            domains: vec!["api.deepseek.com"],
        }
    }

    /// MiniMax characteristics
    pub fn minimax() -> Self {
        Self {
            provider: AiProvider::MiniMax,
            endpoints: vec![
                "/v1/text/chatcompletion_v2",
                "/v1/embeddings",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["group-id"],
            model_patterns: vec!["abab", "abab6", "abab5.5"],
            domains: vec!["api.minimax.chat"],
        }
    }

    /// SenseTime SenseChat/SenseNova (商汤) characteristics
    pub fn sensetime() -> Self {
        Self {
            provider: AiProvider::SenseTime,
            endpoints: vec![
                "/v1/chat/completions",
                "/nova/v1/chat/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["SenseChat", "SenseNova", "sensechat"],
            domains: vec!["api.sensenova.cn"],
        }
    }

    /// iFlytek Spark/Xinghuo (科大讯飞/星火) characteristics
    pub fn iflytek_spark() -> Self {
        Self {
            provider: AiProvider::IFlytekSpark,
            endpoints: vec![
                "/v1.1/chat",
                "/v2.1/chat",
                "/v3.1/chat",
                "/v3.5/chat",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["spark", "general", "generalv2", "generalv3"],
            domains: vec!["spark-api.xf-yun.com", "xfyun.cn"],
        }
    }

    /// 01.AI Yi (零一万物) characteristics
    pub fn zeroone_ai() -> Self {
        Self {
            provider: AiProvider::ZeroOneAI,
            endpoints: vec![
                "/v1/chat/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["yi-34b", "yi-6b", "yi-large"],
            domains: vec!["api.lingyiwanwu.com", "01.ai"],
        }
    }

    /// Baichuan (百川) characteristics
    pub fn baichuan() -> Self {
        Self {
            provider: AiProvider::Baichuan,
            endpoints: vec![
                "/v1/chat/completions",
                "/v1/stream/chat",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec![],
            model_patterns: vec!["baichuan2-", "baichuan-"],
            domains: vec!["api.baichuan-ai.com"],
        }
    }

    // === Other Global Providers ===

    /// Reka AI (Singapore) characteristics
    pub fn reka_ai() -> Self {
        Self {
            provider: AiProvider::RekaAI,
            endpoints: vec![
                "/v1/chat",
                "/v1/completions",
            ],
            auth_headers: vec!["authorization"],
            custom_headers: vec!["x-api-key"],
            model_patterns: vec!["reka-core", "reka-flash", "reka-edge"],
            domains: vec!["api.reka.ai"],
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
        assert!(providers.len() >= 26); // Now includes Chinese and global providers
    }

    #[test]
    fn test_chinese_providers() {
        // Test Alibaba Qwen
        let qwen = ProviderCharacteristics::alibaba_qwen();
        assert_eq!(qwen.provider, AiProvider::AlibabaQwen);
        assert!(qwen.matches_domain("dashscope.aliyuncs.com"));
        assert!(qwen.matches_model("qwen-plus"));

        // Test Baidu ERNIE
        let ernie = ProviderCharacteristics::baidu_ernie();
        assert_eq!(ernie.provider, AiProvider::BaiduErnie);
        assert!(ernie.matches_domain("aip.baidubce.com"));
        assert!(ernie.matches_model("ernie-bot"));

        // Test Zhipu GLM
        let glm = ProviderCharacteristics::zhipu_glm();
        assert_eq!(glm.provider, AiProvider::ZhipuGLM);
        assert!(glm.matches_model("glm-4"));

        // Test Moonshot Kimi
        let kimi = ProviderCharacteristics::moonshot_kimi();
        assert_eq!(kimi.provider, AiProvider::MoonshotKimi);
        assert!(kimi.matches_domain("api.moonshot.cn"));
    }

    #[test]
    fn test_global_providers() {
        // Test Perplexity
        let perplexity = ProviderCharacteristics::perplexity_ai();
        assert_eq!(perplexity.provider, AiProvider::PerplexityAI);
        assert!(perplexity.matches_model("pplx-"));

        // Test xAI
        let xai = ProviderCharacteristics::xai();
        assert_eq!(xai.provider, AiProvider::XAI);
        assert!(xai.matches_model("grok-"));
    }
}
