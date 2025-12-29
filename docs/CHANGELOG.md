# 更新日志

所有重要的项目变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [2.0.1] - 2025-12-29

### 安全修复

- ✅ **深度安全审计修复**: 修复配置隐患和防御纵深改进
  - 修复 TLS 库默认 Feature 配置隐患（高风险）
  - 添加 HTTP/2 和 HTTP/3 Header 压缩炸弹防护
  - 添加 Cookie Secure 属性安全检查
  - 确认 TLS 证书验证默认行为

### 改进

- ✅ **提交前全面测试**: 添加自动测试脚本和 Git pre-commit hook
  - 自动运行代码格式化检查
  - 自动运行编译检查
  - 自动运行 Clippy 检查
  - 自动运行单元测试和集成测试
  - 自动运行安全审计
  - 所有测试通过才能提交

### 修复

- ✅ 修复 Clippy `needless_borrows_for_generic_args` 警告
- ✅ 修复代码格式化问题

---

## [2.0.0] - 2025-12-29

### 重大变更

- ✅ **Workspace 架构重构**: 将单一 crate 重构为 Cargo Workspace 架构
  - 拆分为 7 个独立 crate：fingerprint-core, fingerprint-tls, fingerprint-profiles, fingerprint-headers, fingerprint-http, fingerprint-dns, fingerprint
  - 每个 crate 职责单一，边界清晰
  - 支持并行编译，提高构建速度
  - 更清晰的依赖关系管理

### 改进

- ✅ **模块化设计**: 每个 crate 职责单一，易于维护和扩展
- ✅ **编译优化**: 支持并行编译，只重新编译修改的 crate
- ✅ **依赖管理**: 更清晰的依赖关系，减少不必要的依赖传递
- ✅ **向后兼容**: 主库 API 完全保持不变，用户无需修改代码

### 文档

- ✅ 新增 [WORKSPACE_ARCHITECTURE.md](docs/WORKSPACE_ARCHITECTURE.md) - Workspace 架构详细文档
- ✅ 更新 README.md 说明 Workspace 架构
- ✅ 更新开发流程文档

### 技术细节

- 所有源代码从 `src/` 迁移到 `crates/` 目录
- 更新所有导入路径以使用新的 crate 结构
- 保持所有公共 API 向后兼容
- 所有测试和示例代码无需修改
- 升级到 Rust 1.92.0（最新稳定版）
- 升级 cargo-deny 到 0.18.9（支持 CVSS 4.0）
- 更新 netconnpool 到 v1.0.1
- 修复所有 doctest 导入路径问题
- 全面测试和代码审核完成

---

## [1.0.0] - 2024-12

### 新增
- ✅ 完整的 TLS Client Hello Spec 实现
- ✅ 69 个真实浏览器指纹配置
- ✅ JA4 指纹生成（sorted 和 unsorted 版本）
- ✅ 指纹比较和最佳匹配查找
- ✅ GREASE 值过滤和处理
- ✅ HTTP/2 配置（Settings、Pseudo Header Order、Header Priority）
- ✅ HTTP Headers 生成（30+ 种语言支持）
- ✅ User-Agent 自动匹配
- ✅ 移动端指纹支持（iOS、Android）

### 改进
- ✅ 使用 `TlsVersion` 枚举替代 `u16`，提高类型安全
- ✅ 完整的错误处理
- ✅ 性能优化（字符串分配、排序算法）
- ✅ 代码质量提升（通过所有 Clippy 检查）

### 文档
- ✅ 完整的 README.md
- ✅ API 文档
- ✅ 代码示例
- ✅ 架构文档
- ✅ 优化报告

### 测试
- ✅ 40 个单元测试
- ✅ 27 个集成测试
- ✅ 8 个文档测试
- ✅ 总计 75 个测试全部通过

[1.0.0]: https://github.com/vistone/fingerprint-rust/releases/tag/v1.0.0
