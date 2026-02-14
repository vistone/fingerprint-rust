# ❌ DEPRECATED - 此目录已废弃

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



**废弃日期**: 2026-02-13  
**原因**: API Gateway 应该使用 Rust 实现，不应该使用 Python  
**替代方案**: `crates/fingerprint-gateway/` (Rust + actix-web)

---

## 为什么废弃？

此目录包含 Phase 9.4 的 Python FastAPI 实现（1,879 行代码），这违背了项目的 **纯 Rust** 定位。

**问题**:
1. ❌ 违背项目纯 Rust 定位
2. ❌ 性能劣势（Python ~100ms vs Rust ~10ms）
3. ❌ 资源占用高（Python ~150MB vs Rust ~20MB）
4. ❌ 引入不必要的技术栈混合
5. ❌ `crates/fingerprint-core/src/rate_limiting.rs` 已有 Token Bucket 实现

---

## 新实现

请使用 Rust 实现：

```bash
# 新的 Rust Gateway
cd crates/fingerprint-gateway/

# 运行
cargo run --release
```

**特性**:
- ✅ actix-web 高性能 Web 框架
- ✅ 复用现有的 rate_limiting.rs
- ✅ Redis 后端支持
- ✅ Prometheus 指标监控
- ✅ 10x 性能提升
- ✅ 87% 内存节省

---

## 迁移指南

如果你正在使用 `fingerprint_api/`：

### API 端点映射

```
旧端点 (Python)                         新端点 (Rust)
---------------------------------------- → ------------------------------------------
POST /api/v1/rate-limit/check           → POST /api/v1/rate-limit/check
GET  /api/v1/rate-limit/status          → GET  /api/v1/rate-limit/status
GET  /api/v1/health                     → GET  /api/v1/health
GET  /api/v1/metrics                    → GET  /api/v1/metrics
```

### 请求格式（保持兼容）

```json
// POST /api/v1/rate-limit/check
{
  "api_key": "sk_test_xxx",
  "endpoint": "/api/fingerprint/generate",
  "client_ip": "1.2.3.4"
}
```

### 响应格式（保持兼容）

```json
{
  "allowed": true,
  "quota_tier": "Pro",
  "remaining": 950,
  "limit": 1000,
  "reset_at": "2026-02-13T10:00:00Z"
}
```

---

## 时间线

- **2026-02-13**: fingerprint_api 标记为废弃
- **2026-02-20**: fingerprint-gateway (Rust) 完成开发
- **2026-02-27**: 生产环境迁移完成
- **2026-03-06**: fingerprint_api 归档至 `archive/`

---

## 相关文档

- [全面架构审查报告](../COMPREHENSIVE_ARCHITECTURE_REVIEW.md)
- [Rust Gateway 设计文档](../crates/fingerprint-gateway/README.md)
- [迁移指南](../docs/guides/MIGRATE_FROM_PYTHON_API.md)

---

⚠️ **此目录将在 2026-03-06 移至 `archive/python-experiments/phase-9-4-incorrect/`**

请尽快迁移至 Rust 实现！
