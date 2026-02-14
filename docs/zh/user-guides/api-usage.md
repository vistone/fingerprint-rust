# API调用指南

**版本**: v1.0  
**最后更新**: 2026-02-13  
**适用版本**: fingerprint-rust 2.1.0+

---

## 🎯 概述

本指南详细介绍 fingerprint-rust 项目的API网关和REST API接口使用方法。

## 🏗️ API架构

### 系统架构图
```
┌─────────────┐    ┌──────────────┐    ┌────────────────┐
│    客户端   │───▶│  API网关     │───▶│  指纹服务      │
│   (用户)    │    │ (Kong)       │    │ (fingerprint)  │
└─────────────┘    └──────────────┘    └────────────────┘
                         │
                         ▼
                   ┌──────────────┐
                   │  速率限制    │
                   │  (Redis)     │
                   └──────────────┘
```

### 核心组件
- **API网关**: Kong OSS 3.x
- **认证授权**: JWT + API Key
- **速率限制**: Redis分布式限速
- **负载均衡**: Kubernetes Service
- **监控告警**: Prometheus + Grafana

## 🔐 认证和授权

### API密钥认证

#### 获取API密钥
```bash
# 注册获取API密钥
curl -X POST https://api.fingerprint.example.com/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secure_password"
  }'
```

#### 使用API密钥
```bash
curl -H "apikey: YOUR_API_KEY" \
  https://api.fingerprint.example.com/v1/fingerprints/profiles
```

### JWT令牌认证

#### 获取JWT令牌
```bash
curl -X POST https://api.fingerprint.example.com/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your_username",
    "password": "your_password"
  }'
```

#### 使用JWT令牌
```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  https://api.fingerprint.example.com/v1/fingerprints/generate
```

## 📊 核心API接口

### 1. 指纹管理接口

#### 获取指纹配置列表
```http
GET /v1/fingerprints/profiles
Headers: Authorization: Bearer {token}
```

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.fingerprint.example.com/v1/fingerprints/profiles
```

**响应示例**:
```json
{
  "profiles": [
    {
      "id": "chrome_120_win",
      "name": "Chrome 120 Windows",
      "browser": "Chrome",
      "version": "120.0.0.0",
      "platform": "Windows",
      "supported_protocols": ["http1", "http2", "http3"]
    }
  ],
  "total": 66
}
```

#### 获取特定指纹详情
```http
GET /v1/fingerprints/profiles/{profile_id}
Headers: Authorization: Bearer {token}
```

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.fingerprint.example.com/v1/fingerprints/profiles/chrome_120_win
```

### 2. 指纹生成接口

#### 生成自定义指纹
```http
POST /v1/fingerprints/generate
Headers: 
  Authorization: Bearer {token}
  Content-Type: application/json
```

```bash
curl -X POST https://api.fingerprint.example.com/v1/fingerprints/generate \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "browser": "Chrome",
    "version": "120.0.0.0",
    "platform": "Windows",
    "customizations": {
      "user_agent": "Mozilla/5.0 Custom",
      "locale": "zh-CN",
      "timezone": "Asia/Shanghai"
    }
  }'
```

#### 批量生成指纹
```http
POST /v1/fingerprints/batch-generate
Headers: Authorization: Bearer {token}
```

```bash
curl -X POST https://api.fingerprint.example.com/v1/fingerprints/batch-generate \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "count": 10,
    "profiles": ["chrome_120_win", "firefox_120_win"],
    "distribution": "random"
  }'
```

### 3. 请求代理接口

#### 发送代理请求
```http
POST /v1/proxy/request
Headers: Authorization: Bearer {token}
```

```bash
curl -X POST https://api.fingerprint.example.com/v1/proxy/request \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://httpbin.org/headers",
    "method": "GET",
    "profile": "chrome_120_win",
    "headers": {
      "Custom-Header": "value"
    }
  }'
```

**响应示例**:
```json
{
  "status": 200,
  "headers": {
    "content-type": "application/json",
    "server": "nginx"
  },
  "body": "{\"headers\":{\"Host\":\"httpbin.org\",...}}",
  "timing": {
    "dns_lookup": 15,
    "tcp_connection": 23,
    "tls_handshake": 45,
    "total": 85
  },
  "fingerprint_used": "chrome_120_win"
}
```

### 4. 监控和统计接口

#### 获取使用统计
```http
GET /v1/analytics/usage
Headers: Authorization: Bearer {token}
Query: start_date=2026-01-01&end_date=2026-01-31
```

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  "https://api.fingerprint.example.com/v1/analytics/usage?start_date=2026-01-01&end_date=2026-01-31"
```

#### 获取性能指标
```http
GET /v1/analytics/performance
Headers: Authorization: Bearer {token}
```

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.fingerprint.example.com/v1/analytics/performance
```

## ⚡ 高级功能API

### 1. 指纹池管理

#### 创建指纹池
```http
POST /v1/pools
Headers: Authorization: Bearer {token}
```

```bash
curl -X POST https://api.fingerprint.example.com/v1/pools \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_crawler_pool",
    "profiles": ["chrome_120_win", "firefox_120_win", "safari_17_mac"],
    "strategy": "round_robin",
    "size": 10
  }'
```

#### 使用指纹池发送请求
```http
POST /v1/pools/{pool_id}/request
Headers: Authorization: Bearer {token}
```

### 2. 动态指纹生成

#### 实时指纹生成
```http
POST /v1/fingerprints/dynamic
Headers: Authorization: Bearer {token}
```

```bash
curl -X POST https://api.fingerprint.example.com/v1/fingerprints/dynamic \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "target_url": "https://example.com",
    "adaptation_level": "high",
    "simulation": {
      "mouse_movement": true,
      "typing_patterns": true,
      "viewport_changes": true
    }
  }'
```

### 3. 异常检测和规避

#### 配置异常检测规则
```http
PUT /v1/security/anomaly-detection
Headers: Authorization: Bearer {token}
```

```bash
curl -X PUT https://api.fingerprint.example.com/v1/security/anomaly-detection \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "enabled": true,
    "sensitivity": "medium",
    "bypass_strategies": ["timing_randomization", "behavior_simulation"],
    "custom_rules": [
      {
        "pattern": "cloudflare_challenge",
        "action": "rotate_fingerprint"
      }
    ]
  }'
```

## 📈 速率限制和配额

### 配额层级

| 层级 | 请求限制 | 并发限制 | 特殊权限 |
|------|----------|----------|----------|
| 免费 | 100/min | 5 | 基础功能 |
| 专业 | 1000/min | 50 | 高级功能 |
| 企业 | 无限制 | 200 | 全部功能 |
| 合作伙伴 | 无限制 | 500 | 定制功能 |

### 检查配额使用情况
```http
GET /v1/account/quota
Headers: Authorization: Bearer {token}
```

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.fingerprint.example.com/v1/account/quota
```

**响应示例**:
```json
{
  "current_tier": "professional",
  "limits": {
    "requests_per_minute": 1000,
    "concurrent_requests": 50
  },
  "usage": {
    "current_minute_requests": 456,
    "current_concurrent": 12
  },
  "reset_time": "2026-02-13T17:00:00Z"
}
```

## 🔧 SDK和客户端库

### Python客户端示例
```python
from fingerprint_sdk import FingerprintClient

# 初始化客户端
client = FingerprintClient(
    api_key="YOUR_API_KEY",
    base_url="https://api.fingerprint.example.com"
)

# 获取指纹列表
profiles = client.get_profiles()
print(f"Available profiles: {len(profiles)}")

# 发送代理请求
response = client.proxy_request(
    url="https://httpbin.org/headers",
    profile="chrome_120_win"
)
print(f"Status: {response.status_code}")
```

### JavaScript客户端示例
```javascript
import { FingerprintClient } from '@fingerprint/sdk';

const client = new FingerprintClient({
  apiKey: 'YOUR_API_KEY',
  baseUrl: 'https://api.fingerprint.example.com'
});

// 生成自定义指纹
const fingerprint = await client.generateFingerprint({
  browser: 'Chrome',
  version: '120.0.0.0',
  platform: 'Windows'
});

// 发送请求
const response = await client.proxyRequest({
  url: 'https://httpbin.org/headers',
  profile: 'chrome_120_win'
});
```

### Rust客户端示例
```rust
use fingerprint_client::{Client, ProxyRequest};

let client = Client::new("YOUR_API_KEY")?;

let request = ProxyRequest {
    url: "https://httpbin.org/headers".to_string(),
    profile: Some("chrome_120_win".to_string()),
    method: "GET".to_string(),
    ..Default::default()
};

let response = client.proxy_request(request).await?;
println!("Status: {}", response.status);
```

## 🆘 错误处理

### 常见HTTP状态码

| 状态码 | 含义 | 解决方案 |
|--------|------|----------|
| 200 | 成功 | 正常处理响应 |
| 400 | 请求错误 | 检查请求参数 |
| 401 | 未授权 | 检查认证信息 |
| 403 | 禁止访问 | 检查权限或配额 |
| 429 | 速率限制 | 等待配额重置 |
| 500 | 服务器错误 | 联系技术支持 |

### 错误响应格式
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "请求频率超过限制",
    "details": {
      "limit": 1000,
      "current": 1050,
      "reset_time": "2026-02-13T17:00:00Z"
    }
  }
}
```

## 📚 相关资源

- [完整API参考](../reference/api-reference.md)
- [部署指南](../reference/deployment-manual.md)
- [性能基准](../reference/performance-benchmarks.md)
- [安全配置](security-configuration.md)

---
*最后更新: 2026-02-13*  
*版本: v1.0*