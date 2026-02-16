//! # Global AI Provider Detection Example
//!
//! Demonstrates detection of Chinese and other global AI model providers,
//! not just US-based providers.

use fingerprint_ai_models::{detect_ai_provider, AiProvider};

fn main() {
    println!("=== Global AI Provider Detection Demo ===\n");

    // Chinese Providers Examples
    println!("🇨🇳 Chinese AI Providers\n");
    println!("═══════════════════════════════════════\n");

    // Example 1: Alibaba Qwen (通义千问)
    println!("Example 1: Alibaba Qwen Detection");
    println!("---------------------------------");
    let headers = vec![
        ("Authorization".to_string(), "Bearer sk-xxx".to_string()),
        ("X-DashScope-API-Key".to_string(), "sk-xxx".to_string()),
    ];
    if let Some(result) = detect_ai_provider(
        &headers,
        "/v1/services/aigc/text-generation/generation",
        None,
    ) {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 2: Baidu ERNIE (文心一言)
    println!("Example 2: Baidu ERNIE Bot Detection");
    println!("------------------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer xxx".to_string())];
    if let Some(result) = detect_ai_provider(
        &headers,
        "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/ernie-bot",
        Some("ernie-3.5-8k"),
    ) {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 3: Tencent Hunyuan (混元)
    println!("Example 3: Tencent Hunyuan Detection");
    println!("------------------------------------");
    let headers = vec![
        ("Authorization".to_string(), "Bearer xxx".to_string()),
        ("X-TC-Action".to_string(), "ChatCompletions".to_string()),
    ];
    if let Some(result) =
        detect_ai_provider(&headers, "/hyllm/v1/chat/completions", Some("hunyuan-pro"))
    {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 4: ByteDance Doubao (豆包)
    println!("Example 4: ByteDance Doubao Detection");
    println!("-------------------------------------");
    let headers = vec![
        ("Authorization".to_string(), "Bearer xxx".to_string()),
        ("X-TT-LogId".to_string(), "xxx".to_string()),
    ];
    if let Some(result) =
        detect_ai_provider(&headers, "/api/v3/chat/completions", Some("doubao-pro"))
    {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 5: Zhipu GLM (智谱)
    println!("Example 5: Zhipu AI ChatGLM Detection");
    println!("--------------------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer xxx".to_string())];
    if let Some(result) =
        detect_ai_provider(&headers, "/api/paas/v4/chat/completions", Some("glm-4"))
    {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 6: Moonshot Kimi (月之暗面)
    println!("Example 6: Moonshot AI Kimi Detection");
    println!("--------------------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer sk-xxx".to_string())];
    if let Some(result) =
        detect_ai_provider(&headers, "/v1/chat/completions", Some("moonshot-v1-8k"))
    {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 7: DeepSeek
    println!("Example 7: DeepSeek Detection");
    println!("-----------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer sk-xxx".to_string())];
    if let Some(result) =
        detect_ai_provider(&headers, "/v1/chat/completions", Some("deepseek-chat"))
    {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 8: MiniMax
    println!("Example 8: MiniMax Detection");
    println!("----------------------------");
    let headers = vec![
        ("Authorization".to_string(), "Bearer xxx".to_string()),
        ("Group-ID".to_string(), "xxx".to_string()),
    ];
    if let Some(result) =
        detect_ai_provider(&headers, "/v1/text/chatcompletion_v2", Some("abab6-chat"))
    {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 9: iFlytek Spark (科大讯飞/星火)
    println!("Example 9: iFlytek Spark Detection");
    println!("-----------------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer xxx".to_string())];
    if let Some(result) = detect_ai_provider(&headers, "/v3.5/chat", Some("spark-v3.5")) {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 10: 01.AI Yi (零一万物)
    println!("Example 10: 01.AI Yi Detection");
    println!("-------------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer xxx".to_string())];
    if let Some(result) = detect_ai_provider(&headers, "/v1/chat/completions", Some("yi-large")) {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Other Global Providers
    println!("\n🌍 Other Global Providers\n");
    println!("═══════════════════════════════════════\n");

    // Example 11: Perplexity AI
    println!("Example 11: Perplexity AI Detection");
    println!("-----------------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer pplx-xxx".to_string())];
    if let Some(result) = detect_ai_provider(&headers, "/chat/completions", Some("pplx-70b-online"))
    {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 12: xAI Grok
    println!("Example 12: xAI Grok Detection");
    println!("------------------------------");
    let headers = vec![
        ("Authorization".to_string(), "Bearer xxx".to_string()),
        ("X-API-Key".to_string(), "xxx".to_string()),
    ];
    if let Some(result) = detect_ai_provider(&headers, "/v1/chat/completions", Some("grok-1")) {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 13: Stability AI
    println!("Example 13: Stability AI Detection");
    println!("----------------------------------");
    let headers = vec![
        ("Authorization".to_string(), "Bearer sk-xxx".to_string()),
        ("Stability-Client-ID".to_string(), "xxx".to_string()),
    ];
    if let Some(result) = detect_ai_provider(
        &headers,
        "/v1/generation/stable-diffusion-xl",
        Some("sdxl-1.0"),
    ) {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Example 14: AI21 Labs
    println!("Example 14: AI21 Labs Detection");
    println!("-------------------------------");
    let headers = vec![("Authorization".to_string(), "Bearer xxx".to_string())];
    if let Some(result) = detect_ai_provider(
        &headers,
        "/studio/v1/chat/completions",
        Some("jamba-instruct"),
    ) {
        println!("  Provider: {}", result.provider.as_str());
        println!("  Confidence: {:.2}%", result.confidence * 100.0);
        if let Some(model) = &result.model {
            println!("  Model: {}", model);
        }
    }
    println!();

    // Provider Count Summary
    println!("\n📊 Provider Summary\n");
    println!("═══════════════════════════════════════\n");

    let provider_count = vec![
        ("US/Western Providers", 13),
        ("Chinese Providers", 12),
        ("Other Global Providers", 1),
    ];

    for (category, count) in provider_count {
        println!("  {}: {}", category, count);
    }
    println!("\n  Total Supported Providers: 26+");

    println!("\n✅ Detection Complete!");
}
