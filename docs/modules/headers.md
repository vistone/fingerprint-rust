# Headers 模块文档

## 概述

`headers` 模块提供 HTTP 请求头生成功能，根据浏览器类型生成标准的 HTTP 请求头，支持 30+ 种语言。

## 模块位置

`src/headers.rs`

## 核心类型

### HTTPHeaders

HTTP 请求头结构。

```rust
pub struct HTTPHeaders {
    pub accept: String,
    pub accept_language: String,
    pub accept_encoding: String,
    pub user_agent: String,
    pub sec_ch_ua: Option<String>,
    pub sec_ch_ua_mobile: Option<String>,
    pub sec_ch_ua_platform: Option<String>,
    pub sec_fetch_site: Option<String>,
    pub sec_fetch_mode: Option<String>,
    pub sec_fetch_user: Option<String>,
    pub sec_fetch_dest: Option<String>,
    pub upgrade_insecure_requests: Option<String>,
    // ...
}
```

## 主要函数

### Headers 生成

- `generate_headers(browser_type: BrowserType, user_agent: &str, is_mobile: bool) -> HTTPHeaders`
  - 根据浏览器类型生成标准 HTTP 请求头

- `random_language() -> String`
  - 随机选择语言（30+ 种语言支持）

## 使用示例

```rust
use fingerprint::{HTTPHeaders, BrowserType, random_language};

// 生成标准 HTTP 请求头
let headers = HTTPHeaders::generate_headers(
    BrowserType::Chrome,
    "Mozilla/5.0 ...",
    false
);

// 随机语言
let lang = random_language();
println!("Accept-Language: {}", lang);

// 使用自定义 User-Agent
let headers = HTTPHeaders {
    user_agent: "Custom User-Agent".to_string(),
    ..HTTPHeaders::default()
};
```

## 支持的语言

支持 30+ 种语言的 Accept-Language，包括：
- 中文（简体、繁体）
- 英语（美国、英国）
- 日语、韩语
- 法语、德语、西班牙语
- 俄语、阿拉伯语
- 等等...

## 相关文档

- [User-Agent 模块文档](useragent.md)
- [HTTP 客户端文档](http_client.md)
