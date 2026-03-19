# Vendor 目录说明

本目录存放第三方代码库。

其中一部分仅用于参考研究，另一部分是为了供应链控制而 vendoring 到仓库内的关键依赖。

## rustls

**位置**: `vendor/rustls/`

**来源**: https://github.com/rustls/rustls.git

**说明**: Rust 实现的现代 TLS 库，用于研究如何集成自定义 ClientHello 或改进 TLS 实现。

**主要目录结构**:
- `rustls/` - 核心 rustls 库
- `rustls-ring/` - ring 加密提供者
- `rustls-post-quantum/` - 后量子加密支持
- `examples/` - 示例代码
- `provider-example/` - 自定义提供者示例

**用途**:
- 研究 rustls 的 ClientHello 自定义机制
- 参考 TLS 握手实现
- 了解如何集成自定义 TLS 指纹

**注意**: 
- 此目录已添加到 `.gitignore`，不会被提交到仓库
- 如需更新，请使用 `git pull` 在 `vendor/rustls/` 目录中执行

## netconnpool-rust

**位置**: `vendor/netconnpool-rust/`

**来源**: https://github.com/vistone/netconnpool-rust

**版本策略**:
- 上游 tag: `v1.0.4`
- 当前 vendored commit: `225f7ce4b7a3d1f167efff73eb36aa47d7fd15f2`

**说明**:
- 这是项目 HTTP 连接池路径的关键依赖。
- 为降低 Git 依赖供应链风险，workspace 已改为使用仓库内 path 依赖，而不是在构建时直接拉取远端 Git 源。

**更新流程**:
1. 从上游仓库拉取并审查目标 tag/commit。
2. 更新 `vendor/netconnpool-rust/` 目录内容。
3. 运行 `cargo deny check advisories bans licenses sources`。
4. 运行启用 `connection-pool` 的编译和测试验证。
