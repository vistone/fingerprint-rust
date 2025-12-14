# Vendor 目录说明

本目录存放第三方代码库，用于参考和研究。

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
