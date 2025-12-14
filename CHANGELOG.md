# 更新日志

所有重要的项目变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [1.0.0] - 2024-12

### 新增
- ✅ 完整的 TLS Client Hello Spec 实现
- ✅ 66 个真实浏览器指纹配置
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
