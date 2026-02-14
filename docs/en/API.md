# API Reference Document

**Version**: v1.0  
**Last Updated**: 2026-02-13  
**Document Type**: Technical Reference

---



## Core Functions

### Random Fingerprint Generation

```rust
// Generate a random fingerprint (recommended)
pub fn get_random_fingerprint() -> Result<FingerprintResult, String>
pub fn get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, String>
pub fn get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn Error>>
pub fn get_random_fingerprint_by_browser_with_os(
    browser_type: &str,
    os: Option<OperatingSystem>,
) -> Result<FingerprintResult, Box<dyn Error>>
```

### User-Agent Generation

```rust
pub fn get_user_agent_by_profile_name(profile_name: &str) -> Result<String, String>
pub fn get_user_agent_by_profile_name_with_os(
    profile_name: &str,
    os: OperatingSystem,
) -> Result<String, String>
pub fn random_os() -> OperatingSystem
pub fn random_language() -> String
```

### HTTP Headers Generation

```rust
pub fn generate_headers(
    browser_type: BrowserType,
    user_agent: &str,
    is_mobile: bool,
) -> HTTPHeaders
```

## Data Structures

### FingerprintResult

```rust
pub struct FingerprintResult {
    pub profile: ClientProfile,           // TLS fingerprint configuration
    pub user_agent: String,               // Corresponding User-Agent string
    pub hello_client_id: String,          // ClientHello ID
    pub headers: HTTPHeaders,             // Standard HTTP request headers
}
```

### HTTPHeaders

```rust
pub struct HTTPHeaders {
    pub accept: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub user_agent: String,
    pub sec_fetch_site: String,
    pub sec_fetch_mode: String,
    pub sec_fetch_user: String,
    pub sec_fetch_dest: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_mobile: String,
    pub sec_ch_ua_platform: String,
    pub upgrade_insecure_requests: String,
    pub custom: HashMap<String, String>,  // User-defined headers
}

impl HTTPHeaders {
    pub fn new() -> Self
    pub fn clone(&self) -> Self
    pub fn set(&mut self, key: &str, value: &str)
    pub fn set_headers(&mut self, custom_headers: &[(&str, &str)])
    pub fn to_map(&self) -> HashMap<String, String>
    pub fn to_map_with_custom(&self, custom_headers: &[(&str, &str)]) -> HashMap<String, String>
}
```

### BrowserType

```rust
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Opera,
    Edge,
}

impl BrowserType {
    pub fn from_str(s: &str) -> Option<Self>
    pub fn as_str(&self) -> &'static str
}
```

### OperatingSystem

```rust
pub enum OperatingSystem {
    Windows10,
    Windows11,
    MacOS13,
    MacOS14,
    MacOS15,
    Linux,
    LinuxUbuntu,
    LinuxDebian,
}

impl OperatingSystem {
    pub fn as_str(&self) -> &'static str
}
```

## Usage Examples

### Basic Usage

```rust
use fingerprint::*;

// Get a random fingerprint
let result = get_random_fingerprint()?;
println!("User-Agent: {}", result.user_agent);

// Get Headers as Map
let headers_map = result.headers.to_map();

// Set custom headers
result.headers.set("Cookie", "session_id=abc123");
```

### Specifying Browser Type

```rust
// Generate a random Chrome fingerprint
let result = get_random_fingerprint_by_browser("chrome")?;

// Specify browser and operating system
let result = get_random_fingerprint_by_browser_with_os(
    "firefox",
    Some(OperatingSystem::Windows10),
)?;
```

### User-Agent Generation

```rust
// Get User-Agent by profile name
let ua = get_user_agent_by_profile_name("chrome_120")?;

// Specify operating system
let ua = get_user_agent_by_profile_name_with_os(
    "chrome_120",
    OperatingSystem::MacOS14,
)?;
```

### Headers Management

```rust
use fingerprint::headers::generate_headers;

// Generate headers
let headers = generate_headers(
    BrowserType::Chrome,
    user_agent,
    false, // is_mobile
);

// Set custom headers
headers.set("Cookie", "session_id=abc123");
headers.set_headers(&[
    ("Authorization", "Bearer token"),
    ("X-API-Key", "key"),
]);

// Convert to Map
let headers_map = headers.to_map();
```

### HTTP Client

```rust
use fingerprint::{HttpClient, HttpClientConfig, chrome_133};

// Create client configuration
let config = HttpClientConfig {
    profile: Some(chrome_133()),
    max_redirects: 10,  // Maximum redirect hops
    verify_tls: true,    // Verify TLS certificates
    prefer_http2: true, // Prefer HTTP/2 when available
    ..Default::default()
};

// Create HTTP Client
let client = HttpClient::new(config);

// Send GET request (automatic redirect handling)
let response = client.get("https://example.com")?;

// Send POST request
let response = client.post("https://example.com/api", b"data")?;

// View response
println!("Status Code: {}", response.status_code);
println!("Response Body: {}", response.body_as_string()?);
```

### Connection Pool Support

```rust
use fingerprint::{HttpClient, HttpClientConfig};
use fingerprint::http_client::PoolManagerConfig;

// Configure connection pool
let pool_config = PoolManagerConfig {
    max_connections: 100,
    min_idle: 10,
    enable_reuse: true,
    ..Default::default()
};

// Create HTTP client with connection pool
let client = HttpClient::with_pool(config, pool_config);

// Send requests using connection pool automatically
let response = client.get("http://example.com/")?;

// View connection pool statistics
if let Some(stats) = client.pool_stats() {
    for stat in stats {
        stat.print();
    }
}
```
