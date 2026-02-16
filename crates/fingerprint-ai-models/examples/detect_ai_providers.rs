//! Example: AI Model Provider Fingerprinting
//!
//! Demonstrates detection of various AI model providers from HTTP requests
//! including OpenAI, Anthropic Claude, Google Gemini, and others.

use fingerprint_ai_models::{detect_ai_provider, detect_sdk, AiProvider};

fn main() {
    println!("=== AI Model Provider Fingerprinting Demo ===\n");

    // Example 1: Detect OpenAI from headers
    println!("Example 1: OpenAI GPT-4 Detection");
    println!("---------------------------------");
    let openai_headers = vec![
        (
            "Authorization".to_string(),
            "Bearer sk-proj-abc123...".to_string(),
        ),
        ("OpenAI-Organization".to_string(), "org-xyz".to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
        (
            "User-Agent".to_string(),
            "openai-python/1.12.0 Python/3.11".to_string(),
        ),
    ];

    if let Some(fp) = detect_ai_provider(&openai_headers, "/v1/chat/completions", None) {
        println!("  Provider: {}", fp.provider.as_str());
        println!("  Confidence: {:.2}%", fp.confidence * 100.0);
        if let Some(sdk) = &fp.sdk {
            println!(
                "  SDK: {} {}",
                sdk,
                fp.sdk_version.as_ref().unwrap_or(&"unknown".to_string())
            );
        }
        if let Some(auth) = &fp.auth_method {
            println!("  Auth Method: {}", auth);
        }
        println!();
    }

    // Example 2: Detect Anthropic Claude
    println!("Example 2: Anthropic Claude Detection");
    println!("------------------------------------");
    let anthropic_headers = vec![
        ("x-api-key".to_string(), "sk-ant-api03-xyz...".to_string()),
        ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
        (
            "User-Agent".to_string(),
            "anthropic-sdk-python/0.8.0 Python/3.11".to_string(),
        ),
    ];

    let claude_body = r#"{"model": "claude-3-opus-20240229", "max_tokens": 1024, "messages": [{"role": "user", "content": "Hello"}]}"#;

    if let Some(fp) = detect_ai_provider(&anthropic_headers, "/v1/messages", Some(claude_body)) {
        println!("  Provider: {}", fp.provider.as_str());
        println!("  Confidence: {:.2}%", fp.confidence * 100.0);
        if let Some(model) = &fp.model {
            println!("  Model: {}", model);
        }
        if let Some(sdk) = &fp.sdk {
            println!(
                "  SDK: {} {}",
                sdk,
                fp.sdk_version.as_ref().unwrap_or(&"unknown".to_string())
            );
        }
        for (key, value) in &fp.metadata {
            println!("  {}: {}", key, value);
        }
        println!();
    }

    // Example 3: Detect Google Gemini
    println!("Example 3: Google Gemini Detection");
    println!("----------------------------------");
    let google_headers = vec![
        (
            "Authorization".to_string(),
            "Bearer ya29.xyz...".to_string(),
        ),
        (
            "x-goog-api-client".to_string(),
            "gl-python/3.11 grpc/1.60.0".to_string(),
        ),
        (
            "x-goog-user-project".to_string(),
            "my-project-123".to_string(),
        ),
        ("Content-Type".to_string(), "application/json".to_string()),
    ];

    if let Some(fp) = detect_ai_provider(&google_headers, "/v1/projects/my-project/locations/us-central1/publishers/google/models/gemini-pro:generateContent", None) {
        println!("  Provider: {}", fp.provider.as_str());
        println!("  Confidence: {:.2}%", fp.confidence * 100.0);
        if let Some(auth) = &fp.auth_method {
            println!("  Auth Method: {}", auth);
        }
        for (key, value) in &fp.metadata {
            println!("  {}: {}", key, value);
        }
        println!();
    }

    // Example 4: Detect Azure OpenAI
    println!("Example 4: Azure OpenAI Detection");
    println!("---------------------------------");
    let azure_headers = vec![
        ("api-key".to_string(), "abc123xyz...".to_string()),
        ("api-version".to_string(), "2023-05-15".to_string()),
        ("Content-Type".to_string(), "application/json".to_string()),
    ];

    if let Some(fp) = detect_ai_provider(
        &azure_headers,
        "/openai/deployments/gpt-4-deployment/chat/completions",
        None,
    ) {
        println!("  Provider: {}", fp.provider.as_str());
        println!("  Confidence: {:.2}%", fp.confidence * 100.0);
        if let Some(model) = &fp.model {
            println!("  Deployment: {}", model);
        }
        for (key, value) in &fp.metadata {
            println!("  {}: {}", key, value);
        }
        println!();
    }

    // Example 5: SDK Detection from User-Agent
    println!("Example 5: SDK Detection from User-Agent");
    println!("----------------------------------------");
    let user_agents = vec![
        "openai-python/1.12.0 Python/3.11",
        "openai-node/4.28.0 node/18.0.0",
        "anthropic-sdk-python/0.8.0 Python/3.11",
        "langchain/0.1.0 openai-python/1.0.0 Python/3.10",
        "llama-index/0.9.0 Python/3.11",
        "curl/7.88.1",
    ];

    for ua in &user_agents {
        if let Some((sdk, version)) = detect_sdk(ua) {
            println!("  UA: {}", ua);
            println!(
                "  → SDK: {} (version: {})",
                sdk,
                version.unwrap_or("unknown".to_string())
            );
        }
    }
    println!();

    // Example 6: Multiple Provider Comparison
    println!("Example 6: Provider Comparison");
    println!("-----------------------------");

    let providers = vec![
        AiProvider::OpenAI,
        AiProvider::Anthropic,
        AiProvider::GoogleGemini,
        AiProvider::AzureOpenAI,
        AiProvider::Mistral,
        AiProvider::Cohere,
        AiProvider::HuggingFace,
        AiProvider::AwsBedrock,
    ];

    println!("  Available AI Providers:");
    for provider in providers {
        println!("    • {}", provider.as_str());
    }
    println!();

    // Example 7: Detection from Request Body
    println!("Example 7: Model Detection from Request Body");
    println!("--------------------------------------------");

    let bodies = vec![
        (
            r#"{"model": "gpt-4-turbo-preview", "messages": []}"#,
            "GPT-4 Turbo",
        ),
        (
            r#"{"model": "claude-3-opus-20240229", "max_tokens": 1024}"#,
            "Claude 3 Opus",
        ),
        (
            r#"{"model": "gemini-1.5-pro", "contents": []}"#,
            "Gemini 1.5 Pro",
        ),
        (
            r#"{"model": "mistral-large-latest", "messages": []}"#,
            "Mistral Large",
        ),
    ];

    for (body, name) in bodies {
        let headers = vec![];
        if let Some(fp) = detect_ai_provider(&headers, "/unknown", Some(body)) {
            println!("  {} detected:", name);
            println!("    Provider: {}", fp.provider.as_str());
            println!("    Model: {}", fp.model.unwrap_or("unknown".to_string()));
            println!("    Confidence: {:.2}%", fp.confidence * 100.0);
        }
    }
    println!();

    // Example 8: Bot Detection Patterns
    println!("Example 8: Bot Detection Indicators");
    println!("-----------------------------------");
    println!("  Indicators of automated AI API usage:");
    println!("    • Regular request intervals (< 0.1 variance)");
    println!("    • Outdated SDK versions");
    println!("    • Missing standard browser headers");
    println!("    • Simplified TLS fingerprints");
    println!("    • High request frequency (> 100/min)");
    println!("    • Generic User-Agent (e.g., 'python-requests', 'curl')");
    println!();

    println!("=== Demo Complete ===");
}
