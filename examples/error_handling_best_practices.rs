//! Example demonstrating best practices for error handling in fingerprint-rust
//!
//! This example shows:
//! - Proper Result type usage
//! - Error type definitions with thiserror
//! - Graceful error recovery
//! - Logging and error reporting

use fingerprint::{chrome_133, HttpClient, HttpClientConfig};
use std::time::Duration;
use thiserror::Error;

/// Custom error types for this application
#[derive(Error, Debug)]
enum AppError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
}

/// Configuration validation
fn validate_config(config: &HttpClientConfig) -> Result<(), AppError> {
    if config.timeout.as_secs() == 0 {
        return Err(AppError::ConfigError(
            "Timeout must be greater than 0".to_string(),
        ));
    }

    if config.max_redirects > 20 {
        return Err(AppError::ConfigError(
            "Too many redirects configured".to_string(),
        ));
    }

    Ok(())
}

/// Validate URL before making request
fn validate_url(url: &str) -> Result<(), AppError> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::InvalidUrl(format!(
            "URL must start with http:// or https://, got: {}",
            url
        )));
    }

    if url.len() > 2048 {
        return Err(AppError::InvalidUrl(
            "URL too long (max 2048 characters)".to_string(),
        ));
    }

    Ok(())
}

/// Make HTTP request with proper error handling
fn make_request(url: &str) -> Result<String, AppError> {
    // Validate URL first
    validate_url(url)?;

    // Get browser profile
    let profile = chrome_133();
    println!("✅ Using browser profile: {}", profile.get_client_hello_str());

    // Create configuration
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
        timeout: Duration::from_secs(30),
        max_redirects: 5,
        prefer_http2: true,
        verify_ssl: true,
        ..Default::default()
    };

    // Validate configuration
    validate_config(&config)?;

    // Create HTTP client
    let client = HttpClient::new(config);
    println!("✅ HTTP client created");

    // Make request with timeout handling
    println!("📡 Sending request to: {}", url);

    let response = client
        .get(url)
        .map_err(|e| AppError::HttpError(format!("Request failed: {}", e)))?;

    println!("✅ Response received:");
    println!("   Status: {}", response.status_code);
    println!("   Version: {}", response.http_version);
    println!("   Body size: {} bytes", response.body.len());

    // Validate response
    if response.status_code != 200 {
        return Err(AppError::HttpError(format!(
            "Non-200 status code: {}",
            response.status_code
        )));
    }

    // Convert body to string with proper error handling
    let body = String::from_utf8(response.body)
        .map_err(|e| AppError::HttpError(format!("Invalid UTF-8 in response: {}", e)))?;

    Ok(body)
}

/// Retry logic with exponential backoff
fn make_request_with_retry(url: &str, max_retries: u32) -> Result<String, AppError> {
    let mut last_error = None;

    for attempt in 1..=max_retries {
        println!("\n🔄 Attempt {}/{}", attempt, max_retries);

        match make_request(url) {
            Ok(body) => {
                println!("✅ Request succeeded on attempt {}", attempt);
                return Ok(body);
            }
            Err(e) => {
                eprintln!("⚠️  Attempt {} failed: {}", attempt, e);
                last_error = Some(e);

                if attempt < max_retries {
                    let delay = Duration::from_secs(2u64.pow(attempt - 1));
                    println!("⏳ Waiting {:?} before retry...", delay);
                    std::thread::sleep(delay);
                }
            }
        }
    }

    Err(last_error.unwrap())
}

/// Process multiple URLs with error handling
fn process_urls(urls: &[&str]) -> Vec<Result<String, AppError>> {
    urls.iter()
        .map(|url| {
            println!("\n{'=':<50}");
            println!("Processing: {}", url);
            println!("{'=':<50}");
            make_request(url)
        })
        .collect()
}

fn main() -> Result<(), AppError> {
    println!("🦀 Error Handling Best Practices Example\n");

    // Example 1: Simple request with error handling
    println!("{'=':<70}");
    println!(" Example 1: Basic Request with Error Handling");
    println!("{'=':<70}");

    match make_request("https://httpbin.org/get") {
        Ok(body) => {
            println!("✅ Success! Body length: {} bytes", body.len());
            println!("First 100 chars: {}", &body[..body.len().min(100)]);
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            eprintln!("Error type: {:?}", e);
        }
    }

    // Example 2: Request with retry logic
    println!("\n{'=':<70}");
    println!(" Example 2: Request with Retry Logic");
    println!("{'=':<70}");

    match make_request_with_retry("https://httpbin.org/get", 3) {
        Ok(_) => println!("✅ Request succeeded with retry"),
        Err(e) => eprintln!("❌ All retries failed: {}", e),
    }

    // Example 3: Multiple URLs with error collection
    println!("\n{'=':<70}");
    println!(" Example 3: Processing Multiple URLs");
    println!("{'=':<70}");

    let urls = vec![
        "https://httpbin.org/get",
        "https://httpbin.org/status/404", // Will fail
        "https://httpbin.org/delay/1",
        "invalid-url",                    // Will fail validation
    ];

    let results = process_urls(&urls);

    // Report results
    println!("\n📊 Summary:");
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    println!("✅ Successful: {}", successes);
    println!("❌ Failed: {}", failures);

    // Detail failures
    if failures > 0 {
        println!("\n❌ Failed requests:");
        for (i, result) in results.iter().enumerate() {
            if let Err(e) = result {
                println!("   - URL {}: {}", i + 1, e);
            }
        }
    }

    // Example 4: Error recovery and fallback
    println!("\n{'=':<70}");
    println!(" Example 4: Error Recovery and Fallback");
    println!("{'=':<70}");

    let primary_url = "https://httpbin.org/status/500"; // Will fail
    let fallback_url = "https://httpbin.org/get"; // Should succeed

    let result = make_request(primary_url).or_else(|e| {
        eprintln!("⚠️  Primary request failed: {}", e);
        println!("🔄 Trying fallback URL...");
        make_request(fallback_url)
    });

    match result {
        Ok(_) => println!("✅ Request succeeded (possibly with fallback)"),
        Err(e) => eprintln!("❌ Both primary and fallback failed: {}", e),
    }

    println!("\n🎉 Example completed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        let result = validate_url("ftp://example.com");
        assert!(result.is_err());
        if let Err(AppError::InvalidUrl(msg)) = result {
            assert!(msg.contains("http://"));
        }
    }

    #[test]
    fn test_validate_url_too_long() {
        let long_url = format!("https://{}.com", "a".repeat(2500));
        let result = validate_url(&long_url);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_zero_timeout() {
        let config = HttpClientConfig {
            timeout: Duration::from_secs(0),
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_too_many_redirects() {
        let config = HttpClientConfig {
            max_redirects: 100,
            timeout: Duration::from_secs(30),
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_error_display() {
        let error = AppError::Timeout(Duration::from_secs(30));
        assert_eq!(error.to_string(), "Timeout after 30s");
    }
}
