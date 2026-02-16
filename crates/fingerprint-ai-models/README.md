# fingerprint-ai-models

AI Model Provider Fingerprinting Library

## Overview

`fingerprint-ai-models` provides comprehensive fingerprinting capabilities for detecting and identifying requests to various generative AI model providers. Based on the latest research and patterns from 2025-2026, this library can identify AI API usage from OpenAI, Anthropic Claude, Google Gemini, Azure OpenAI, and many other providers.

## Features

- **Provider Detection**: Identify major AI providers (OpenAI, Anthropic, Google, Azure, Mistral, Cohere, etc.)
- **HTTP Header Analysis**: Detect provider-specific authentication and versioning headers
- **TLS Fingerprinting**: Match JA3 patterns for different AI service infrastructure
- **SDK Detection**: Identify client libraries (Python, Node.js, JavaScript, etc.) and their versions
- **API Pattern Analysis**: Recognize endpoint structures and request patterns
- **Model Version Detection**: Identify specific AI models (GPT-4, Claude 3, Gemini, etc.)
- **Bot Detection**: Analyze request patterns to identify automated AI API usage

## Supported Providers

- **OpenAI**: GPT-3.5, GPT-4, GPT-4 Turbo, and other OpenAI models
- **Anthropic**: Claude 2, Claude 3 (Haiku, Sonnet, Opus)
- **Google**: Gemini Pro, Gemini Ultra, Gemini 1.5
- **Azure OpenAI**: Azure-hosted OpenAI models
- **Mistral AI**: Mistral Tiny, Small, Medium, Large
- **Cohere**: Command, Command-R, Embed models
- **Meta Llama**: Via various hosting providers
- **Hugging Face**: Inference API
- **AWS Bedrock**: Claude, Titan, J2 models

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
```

## Detection Methods

### 1. HTTP Header Analysis

Identifies providers through:
- Authentication headers (`Authorization`, `x-api-key`, `api-key`)
- Provider-specific headers (`OpenAI-Organization`, `anthropic-version`, `x-goog-api-client`)
- API versioning headers

### 2. Endpoint Pattern Matching

Recognizes API endpoints:
- OpenAI: `/v1/chat/completions`, `/v1/completions`
- Anthropic: `/v1/messages`
- Google: `/v1/projects/.../models/...`
- Azure: `/openai/deployments/...`

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

## Bot Detection Features

The library includes patterns for detecting automated AI API usage:

- Regular request intervals (low variance)
- Outdated SDK versions
- Missing standard browser headers
- Simplified TLS fingerprints
- High request frequency patterns
- Generic User-Agent strings

## Examples

See `examples/detect_ai_providers.rs` for comprehensive usage examples:

```bash
cargo run --package fingerprint-ai-models --example detect_ai_providers
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
- Edge cases

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
