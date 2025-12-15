# 架构设计文档

## 模块结构

本项目采用组件化设计，严格遵循 Rust 语言标准和最佳实践。

### 核心模块

1. **types.rs** - 类型定义模块
   - `BrowserType`: 浏览器类型枚举
   - `OperatingSystem`: 操作系统类型枚举
   - `UserAgentTemplate`: User-Agent 模板结构

2. **utils.rs** - 工具函数模块
   - `random_choice`: 线程安全的随机选择
   - `extract_chrome_version`: 从 User-Agent 提取 Chrome 版本
   - `extract_platform`: 从 User-Agent 提取平台信息
   - `infer_browser_from_profile_name`: 从 profile 名称推断浏览器类型
   - `is_mobile_profile`: 判断是否为移动端 profile

3. **headers.rs** - HTTP Headers 模块
   - `HTTPHeaders`: HTTP 请求头结构
   - `generate_headers`: 根据浏览器类型生成标准 Headers
   - `random_language`: 随机选择语言

4. **useragent.rs** - User-Agent 生成模块
   - `UserAgentGenerator`: User-Agent 生成器
   - `get_user_agent_by_profile_name`: 根据 profile 名称获取 User-Agent
   - `random_os`: 随机选择操作系统

5. **profiles.rs** - 指纹配置模块
   - `ClientProfile`: TLS 指纹配置结构
   - `mapped_tls_clients`: 全局指纹配置映射表
   - 各种浏览器的指纹配置函数

6. **random.rs** - 随机指纹生成模块
   - `FingerprintResult`: 指纹结果结构
   - `get_random_fingerprint`: 随机获取指纹
   - `get_random_fingerprint_by_browser`: 根据浏览器类型获取指纹

7. **tls_config/** - TLS 配置模块
   - `ClientHelloSpec`: TLS ClientHello 规范
   - `ClientHelloSpecBuilder`: Builder 模式构建器
   - `Ja4Fingerprint`: JA4 指纹生成
   - `compare_specs`: 指纹比较功能

8. **tls_handshake/** - TLS 握手模块
   - `TLSHandshakeBuilder`: TLS 握手构建器
   - 支持自定义 ClientHello 消息构建

9. **http_client/** - HTTP 客户端模块
   - `HttpClient`: HTTP 客户端主类
   - `HttpClientConfig`: 客户端配置
   - `http1.rs`: HTTP/1.1 实现
   - `http2.rs`: HTTP/2 实现（多路复用、HPACK）
   - `http3.rs`: HTTP/3 实现（QUIC 协议）
   - `rustls_client_hello_customizer.rs`: 通过 ClientHelloCustomizer 应用浏览器指纹
   - `pool.rs`: 连接池管理（与 netconnpool 集成）

10. **dns/** - DNS 预解析模块（可选，需要 `dns` feature）
    - `Service`: DNS 服务主接口（start/stop）
    - `DNSResolver`: DNS 解析器（高并发查询）
    - `ServerPool`: DNS 服务器池管理
    - `ServerCollector`: DNS 服务器收集器
    - `IPInfoClient`: IP 地理信息客户端
    - `storage`: 多格式数据存储（JSON/YAML/TOML）

## 设计原则

### 1. 职责单一
- 每个模块只负责一个明确的功能领域
- 模块之间保持相互独立
- 仅在业务整合层（random.rs）进行组合

### 2. 输入输出清晰
- 每个函数都有明确的输入参数和返回值
- 使用 Rust 的类型系统确保类型安全
- 错误处理使用 `Result` 类型

### 3. 避免不必要的嵌套与耦合
- 模块之间通过公共接口交互
- 使用 trait 和枚举实现多态
- 避免深层嵌套结构

### 4. 线程安全
- 使用 `OnceLock` 实现线程安全的单例
- 随机数生成使用线程本地随机数生成器
- 所有公共 API 都是线程安全的

### 5. 性能优化
- 使用 `HashMap` 进行快速查找
- 避免不必要的克隆
- 使用引用传递减少内存分配

## 文件组织

```
src/
├── lib.rs              # 库入口，导出公共 API
├── types.rs            # 类型定义
├── utils.rs            # 工具函数
├── headers.rs          # HTTP Headers
├── useragent.rs        # User-Agent 生成
├── random.rs           # 随机指纹生成
├── profiles.rs         # 指纹配置
├── http2_config.rs     # HTTP/2 配置
├── tls_config/         # TLS 配置模块
│   ├── mod.rs
│   ├── spec.rs
│   ├── builder.rs
│   ├── ja4.rs
│   └── ...
├── tls_handshake/      # TLS 握手模块
│   ├── mod.rs
│   ├── builder.rs
│   └── ...
├── http_client/        # HTTP 客户端模块
│   ├── mod.rs
│   ├── http1.rs
│   ├── http2.rs
│   ├── http3.rs
│   ├── rustls_client_hello_customizer.rs
│   └── ...
├── dicttls/            # TLS 字典模块
│   ├── mod.rs
│   ├── cipher_suites.rs
│   └── ...
└── dns/                # DNS 预解析模块（可选）
    ├── mod.rs
    ├── service.rs
    ├── resolver.rs
    └── ...

tests/
├── integration_test.rs
├── http_client_test.rs
├── dns_service_test.rs
└── ...

examples/
├── basic.rs
├── dns_service.rs
└── ...

docs/
├── API.md
├── ARCHITECTURE.md
└── modules/
```

## 测试策略

### 单元测试
- 每个模块都包含单元测试
- 测试覆盖核心功能
- 使用 `#[cfg(test)]` 标记测试代码

### 集成测试
- `tests/integration_test.rs` 包含全面的集成测试
- 测试所有公共 API
- 测试并发安全性
- 测试边界情况

### 测试覆盖
- ✅ 随机指纹获取
- ✅ 指定浏览器类型获取指纹
- ✅ User-Agent 生成
- ✅ HTTP Headers 生成和管理
- ✅ 并发访问安全性
- ✅ 错误处理

## 性能考虑

1. **零分配操作**: 关键路径避免不必要的内存分配
2. **快速查找**: 使用 HashMap 进行 O(1) 查找
3. **线程安全**: 使用线程本地随机数生成器，避免锁竞争
4. **延迟初始化**: 使用 `OnceLock` 实现延迟初始化

## 扩展性

项目设计支持以下扩展：

1. **添加新浏览器指纹**: 在 `profiles.rs` 中添加新的配置函数
2. **添加新 User-Agent 模板**: 在 `useragent.rs` 的 `init_templates` 中添加
3. **添加新语言**: 在 `headers.rs` 的 `LANGUAGES` 数组中添加
4. **添加新操作系统**: 在 `types.rs` 的 `OperatingSystem` 枚举中添加
