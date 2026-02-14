# 实现指南

本目录包含fingerprint-rust（v2.0 - 整合版）的实用实现指南。

## 可用指南

### 协议集成
- **[HTTP/2集成指南](HTTP2_INTEGRATION_GUIDE.md)** - HTTP/2协议集成和优化
- **[DNS集成指南](DNS_INTEGRATION_GUIDE.md)** - DNS预解析和缓存设置

### 指纹识别技术
- **[捕获浏览器指纹](CAPTURE_BROWSER_FINGERPRINTS.md)** 
  - 如何捕获和分析浏览器指纹
  - Firefox特定技术（来自FIREFOX_CAPTURE_GUIDE.md）

- **[TCP指纹指南](TCP_FINGERPRINT.md)** 
  - TCP级别指纹应用和同步
  - 应用示例和最佳实践（来自TCP_FINGERPRINT_APPLICATION.md和TCP_FINGERPRINT_SYNC.md）

- **[统一指纹](UNIFIED_FINGERPRINT.md)** 
  - 统一指纹识别方法和实现
  - 代码示例和用例（来自UNIFIED_FINGERPRINT_EXAMPLE.md）

### 运维和验证
- **[使用指南](USAGE_GUIDE.md)** - 通用使用指南和最佳实践
- **[运营手册](OPERATIONS_RUNBOOK.md)** - 生产环境运维和故障排除

## 整合总结

✅ **已合并**类似指南以实现更好的组织：
- CAPTURE_BROWSER_FINGERPRINTS + FIREFOX_CAPTURE_GUIDE
- TCP_FINGERPRINT_APPLICATION + TCP_FINGERPRINT_SYNC  
- UNIFIED_FINGERPRINT + UNIFIED_FINGERPRINT_EXAMPLE

✅ **已移除**重复项：
- ORGANIZATION_GUIDE.md（请使用[docs/ORGANIZATION.md](../ORGANIZATION.md)）

📁 **从12个文件减少到8个文件** - 文档更聚焦

## 历史指南

归档的阶段性和实验性指南可以在[../archives/historical-guides/](../archives/historical-guides/)中找到。

---

**版本**: 2.0（整合版）  
**最后更新**: 2026-02-14  
**状态**: 积极维护中
