# fingerprint-ai-models

AI Model Provider Fingerprinting and Content Detection Library

## Overview

`fingerprint-ai-models` provides comprehensive fingerprinting capabilities for:
1. **API Fingerprinting**: Detecting and identifying requests to various generative AI model providers
2. **Content Fingerprinting**: Detecting and analyzing AI-generated text content

Based on the latest research and patterns from 2025-2026, this library can identify AI API usage from OpenAI, Anthropic Claude, Google Gemini, Azure OpenAI, and many other providers, as well as detect whether content was AI-generated and which model likely produced it.

## Features

### API Fingerprinting

- **Provider Detection**: Identify major AI providers (OpenAI, Anthropic, Google, Azure, Mistral, Cohere, etc.)
- **HTTP Header Analysis**: Detect provider-specific authentication and versioning headers
- **TLS Fingerprinting**: Match JA3 patterns for different AI service infrastructure
- **SDK Detection**: Identify client libraries (Python, Node.js, JavaScript, etc.) and their versions
- **API Pattern Analysis**: Recognize endpoint structures and request patterns
- **Model Version Detection**: Identify specific AI models (GPT-4, Claude 3, Gemini, etc.)
- **Bot Detection**: Analyze request patterns to identify automated AI API usage

### Content Fingerprinting (NEW!)

- **AI Detection**: Determine if text content was AI-generated
- **Perplexity Analysis**: Measure text predictability (lower = more AI-like)
- **Burstiness Metrics**: Analyze sentence length variance (lower = more AI-like)
- **Vocabulary Analysis**: Type-token ratio and richness metrics
- **Pattern Detection**: Identify AI-characteristic phrases and structures
- **Model Attribution**: Probabilistically identify which AI model generated content (GPT, Claude, Gemini)
- **Confidence Scoring**: Multi-factor confidence calculation

## Supported Providers

### US/Western Providers
- **OpenAI**: GPT-3.5, GPT-4, GPT-4 Turbo, and other OpenAI models
- **Anthropic**: Claude 2, Claude 3 (Haiku, Sonnet, Opus)
- **Google**: Gemini Pro, Gemini Ultra, Gemini 1.5
- **Azure OpenAI**: Azure-hosted OpenAI models
- **Mistral AI**: Mistral Tiny, Small, Medium, Large
- **Cohere**: Command, Command-R, Embed models
- **Meta Llama**: Via various hosting providers
- **Hugging Face**: Inference API
- **AWS Bedrock**: Claude, Titan, J2 models
- **Perplexity AI**: PPLX models, Sonar
- **xAI**: Grok-1, Grok-2
- **Replicate**: Various open-source models
- **Stability AI**: Stable Diffusion, SDXL, SD3
- **AI21 Labs**: Jurassic-2, Jamba

### Chinese Providers (中国AI模型)
- **Alibaba Cloud**: Qwen/Tongyi Qianwen (通义千问)
- **Baidu**: ERNIE Bot/Wenxin Yiyan (文心一言)
- **Tencent**: Hunyuan (混元)
- **ByteDance**: Doubao/Coze (豆包)
- **Zhipu AI**: ChatGLM/GLM-4 (智谱)
- **Moonshot AI**: Kimi (月之暗面)
- **DeepSeek**: DeepSeek Chat, DeepSeek Coder
- **MiniMax**: Abab models
- **SenseTime**: SenseChat/SenseNova (商汤)
- **iFlytek**: Spark/Xinghuo (科大讯飞/星火)
- **01.AI**: Yi models (零一万物)
- **Baichuan**: Baichuan-2 (百川)

### Other Global Providers
- **Reka AI**: Reka Core, Flash, Edge (Singapore)

## Usage

```rust
use fingerprint_ai_models::{detect_ai_provider, detect_sdk, AiProvider};

// Detect provider from HTTP headers
let headers = vec![
    ("Authorization".to_string(), "Bearer sk-...".to_string()),
    ("OpenAI-Organization".to_string(), "org-123".to_string()),
];

if let Some(fp) = detect_ai_provider(&headers, "/v1/chat/completions", None) {
    println!("Provider: {}", fp.provider.as_str());
    println!("Confidence: {:.2}%", fp.confidence * 100.0);
}

// Detect SDK from User-Agent
if let Some((sdk, version)) = detect_sdk("openai-python/1.12.0 Python/3.11") {
    println!("SDK: {} v{}", sdk, version.unwrap_or("unknown".to_string()));
}

// Detect Chinese providers
// Example: Alibaba Qwen
let qwen_headers = vec![
    ("Authorization".to_string(), "Bearer sk-xxx".to_string()),
    ("X-DashScope-API-Key".to_string(), "sk-xxx".to_string()),
];
if let Some(fp) = detect_ai_provider(&qwen_headers, "/v1/services/aigc/text-generation/generation", Some("qwen-plus")) {
    println!("Provider: {}", fp.provider.as_str()); // alibaba_qwen
}

// Example: Baidu ERNIE
let ernie_headers = vec![
    ("Authorization".to_string(), "Bearer xxx".to_string()),
];
if let Some(fp) = detect_ai_provider(&ernie_headers, "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/ernie-bot", Some("ernie-3.5")) {
    println!("Provider: {}", fp.provider.as_str()); // baidu_ernie
}

// Example: Moonshot Kimi
let kimi_headers = vec![
    ("Authorization".to_string(), "Bearer sk-xxx".to_string()),
];
if let Some(fp) = detect_ai_provider(&kimi_headers, "/v1/chat/completions", Some("moonshot-v1-8k")) {
    println!("Provider: {}", fp.provider.as_str()); // moonshot_kimi
}
```

## Detection Methods

### 1. HTTP Header Analysis

Identifies providers through:
- Authentication headers (`Authorization`, `x-api-key`, `api-key`)
- Provider-specific headers (`OpenAI-Organization`, `anthropic-version`, `x-goog-api-client`)
- API versioning headers

### 2. Endpoint Pattern Matching

Recognizes API endpoints:
- **OpenAI**: `/v1/chat/completions`, `/v1/completions`
- **Anthropic**: `/v1/messages`
- **Google**: `/v1/projects/.../models/...`
- **Azure**: `/openai/deployments/...`
- **Alibaba Qwen**: `/v1/services/aigc/text-generation/generation`
- **Baidu ERNIE**: `/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/...`
- **Moonshot**: `/v1/chat/completions`
- **DeepSeek**: `/v1/chat/completions`

### 3. Request Body Analysis

Extracts information from:
- Model names in request body
- Provider-specific request fields
- API parameter patterns

### 4. SDK Detection

Identifies client libraries from User-Agent:
- Official SDKs (openai-python, anthropic-sdk, etc.)
- Framework integrations (LangChain, LlamaIndex)
- Generic HTTP clients (curl, requests, aiohttp)

### 5. TLS Fingerprinting

Matches TLS/JA3 patterns for:
- Infrastructure identification (Fastly, AWS, Google Cloud)
- Bot vs. human distinction
- Automation detection

### 6. Content Detection Methods

Analyzes text content through multiple signals:

#### Perplexity Analysis
Measures how predictable the text is:
- Low perplexity (< 0.3) = highly predictable = AI-like
- High perplexity (> 0.7) = less predictable = human-like

#### Burstiness Metrics
Analyzes sentence length variance:
- Low burstiness (< 0.3) = uniform lengths = AI-like
- High burstiness (> 0.6) = varied lengths = human-like

#### Vocabulary Richness
Type-token ratio (unique words / total words):
- AI typically shows moderate richness (0.3-0.6)
- Very high or very low suggests human writing

#### Pattern Detection
Identifies AI-characteristic elements:
- Repetitive sentence structures
- Formal language overuse ("furthermore", "moreover")
- Characteristic AI phrases ("it's important to note", "delve into")
- Perfect grammar and punctuation
- Predictable transitions

#### Model Attribution
Probabilistically identifies the source model:
- **GPT models**: Verbose, uses specific phrases ("delve into", "it's important to note")
- **Claude**: More structured and formal, consistent formatting
- **Gemini**: Tends to be more concise and direct

## Bot Detection Features

The library includes patterns for detecting automated AI API usage:

- Regular request intervals (low variance)
- Outdated SDK versions
- Missing standard browser headers
- Simplified TLS fingerprints
- High request frequency patterns
- Generic User-Agent strings

## Examples

### API Detection
See `examples/detect_ai_providers.rs` for comprehensive API detection examples:

```bash
cargo run --package fingerprint-ai-models --example detect_ai_providers
```

### Content Detection
See `examples/detect_ai_content.rs` for content analysis examples:

```bash
cargo run --package fingerprint-ai-models --example detect_ai_content
```

## Testing

Run the test suite:

```bash
cargo test --package fingerprint-ai-models
```

All tests include comprehensive coverage of:
- Provider detection accuracy
- Header parsing
- SDK version extraction
- Pattern matching
- Content analysis (perplexity, burstiness, patterns)
- Model attribution
- Edge cases

## Use Cases

### API Monitoring
- Traffic analysis and monitoring
- Compliance and auditing
- Rate limiting and quotas
- Security research

### Content Analysis
- Academic integrity checking
- Content authenticity verification
- Plagiarism detection enhancement
- AI disclosure compliance
- Content moderation assistance

## Security Considerations

This library is designed for:
- Traffic analysis and monitoring
- Compliance and auditing
- Rate limiting and quotas
- Security research

**Note**: Always respect API terms of service and privacy regulations when using fingerprinting techniques.

## References

- [OpenAI API Documentation](https://platform.openai.com/docs)
- [Anthropic Claude API](https://docs.anthropic.com)
- [Google Vertex AI](https://cloud.google.com/vertex-ai)
- [JA3 TLS Fingerprinting](https://github.com/salesforce/ja3)

## License

BSD-3-Clause

## Contributing

Contributions are welcome! Please ensure:
- All tests pass
- New providers include comprehensive tests
- Documentation is updated
- Code follows Rust best practices
