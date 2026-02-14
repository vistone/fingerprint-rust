# Phase 9.4 快速启动指南

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



## 🚀 快速开始

本指南帮助您快速启动 Phase 9.4 速率限制系统并运行负载测试。

---

## 前置条件检查

运行系统验证脚本：

```bash
bash scripts/verify_load_test_ready.sh
```

该脚本会检查：
- ✓ Python 3.7+ 
- ✓ Redis 服务
- ✓ Python 依赖
- ✓ 负载测试工具 (k6, Apache Bench)
- ✓ 测试脚本和负载文件

---

## 步骤 1: 安装依赖

### 1.1 创建虚拟环境 (推荐)

```bash
# 创建虚拟环境
python3 -m venv venv

# 激活虚拟环境
source venv/bin/activate  # Linux/macOS
# 或
venv\Scripts\activate     # Windows
```

### 1.2 安装 Python 依赖

```bash
pip install -r fingerprint_api/requirements.txt
```

**关键依赖**:
- `fastapi==0.104.1` - Web 框架
- `uvicorn==0.24.0` - ASGI 服务器
- `aioredis==2.0.1` - 异步 Redis 客户端
- `redis==5.0.1` - Redis Python 客户端
- `pytest==7.4.3` - 测试框架

### 1.3 安装负载测试工具 (可选)

**k6** (推荐用于综合负载测试):
```bash
# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg \
  --keyserver hkp://keyserver.ubuntu.com:80 \
  --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | \
  sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# macOS
brew install k6

# Docker
docker pull grafana/k6
```

**Apache Bench** (快速基准测试):
```bash
# Ubuntu/Debian
sudo apt-get install apache2-utils

# macOS (预装)
# 已包含在系统中
```

### 1.4 启动 Redis (如果未运行)

```bash
# 检查 Redis 是否运行
redis-cli ping

# 如果未运行，启动 Redis
redis-server

# 或在后台运行
redis-server --daemonize yes
```

---

## 步骤 2: 启动 FastAPI 应用

### 2.1 开发模式 (自动重载)

```bash
uvicorn fingerprint_api.main:app --host 0.0.0.0 --port 8000 --reload
```

**访问地址**:
- API: `http://localhost:8000`
- 交互式文档: `http://localhost:8000/docs`
- API文档: `http://localhost:8000/redoc`
- 健康检查: `http://localhost:8000/health`

### 2.2 生产模式 (多worker)

```bash
uvicorn fingerprint_api.main:app \
  --host 0.0.0.0 \
  --port 8000 \
  --workers 4 \
  --log-level info
```

### 2.3 验证应用启动

```bash
# 健康检查
curl http://localhost:8000/health

# 速率限制状态
curl http://localhost:8000/api/v1/rate-limit/status

# Prometheus 指标
curl http://localhost:8000/api/v1/rate-limit/metrics
```

---

## 步骤 3: 运行负载测试

### 3.1 快速测试 (Apache Bench)

在新终端中运行：

```bash
# 确保应用正在运行
bash tests/load/ab_rate_limiting_test.sh
```

**测试场景**:
1. Free层单用户 (100 req/min)
2. Pro层单用户 (1000 req/min)
3. 10个并发用户 (独立配额)
4. Compare端点 (2x成本)
5. IP限速 (无认证)
6. 健康检查豁免
7. 持续负载 (令牌填充)

**预期结果**:
```
测试1: Free层速率限制
✓ 测试1通过: 120/150 成功

测试2: Pro层速率限制
✓ 测试2通过: 1150/1200 成功

...

=== 所有测试完成 ===
结果保存在: ./load_test_results
```

### 3.2 综合测试 (k6)

#### 基础负载测试 (5分钟)

```bash
k6 run tests/load/k6_rate_limiting_test.js
```

**配置**:
- 预热: 30s → 10 VUs
- 爬坡: 1m → 50 VUs
- 持续: 3m @ 50 VUs
- 减速: 30s → 0 VUs

**自定义参数**:
```bash
# 高负载测试
k6 run --vus 100 --duration 10m tests/load/k6_rate_limiting_test.js

# 指定API端点
k6 run --env API_URL=http://localhost:8000 tests/load/k6_rate_limiting_test.js

# 导出结果
k6 run --out json=results.json tests/load/k6_rate_limiting_test.js
```

#### 突发测试场景

```bash
k6 run --env SCENARIO=burst tests/load/k6_rate_limiting_test.js
```

测试 1.5x 突发容量是否正常工作。

#### 层级对比测试

```bash
k6 run --env SCENARIO=tier_comparison tests/load/k6_rate_limiting_test.js
```

验证 Free/Pro/Enterprise 三个层级的配额独立性。

### 3.3 查看测试结果

```bash
# Apache Bench 结果
ls -lh load_test_results/

# 查看特定测试
cat load_test_results/test1_free_tier.txt

# k6 结果
cat summary.json | jq .
```

---

## 步骤 4: 监控与调试

### 4.1 实时监控

**Prometheus 指标**:
```bash
# 获取所有指标
curl http://localhost:8000/api/v1/rate-limit/metrics

# 核心指标
curl http://localhost:8000/api/v1/rate-limit/metrics | grep rate_limit_total_requests
curl http://localhost:8000/api/v1/rate-limit/metrics | grep rate_limit_rejected_total
curl http://localhost:8000/api/v1/rate-limit/metrics | grep cache_hit_ratio
```

**查询用户配额**:
```bash
# 查看特定用户配额
curl http://localhost:8000/api/v1/rate-limit/quota/test_user_free

# 响应示例
{
  "user_id": "test_user_free",
  "tier": "free",
  "limit_per_minute": 100,
  "available_tokens": 45.2,
  "monthly_quota": 50000,
  "requests_this_month": 287
}
```

**重置配额 (管理员)**:
```bash
curl -X POST http://localhost:8000/api/v1/rate-limit/quota/test_user_free/reset
```

### 4.2 日志查看

```bash
# FastAPI 应用日志
# (如果使用 uvicorn 启动，日志会输出到终端)

# 查看速率限制事件
curl http://localhost:8000/api/v1/rate-limit/events | jq .
```

### 4.3 Redis 调试

```bash
# 连接 Redis
redis-cli

# 查看所有速率限制键
redis> KEYS rl:*

# 查看特定用户配额
redis> GET rl:quota:test_user_free

# 查看所有指标
redis> KEYS rl:metric:*
```

---

## 常见问题

### 问题 1: FastAPI 应用启动失败

**错误**: `ModuleNotFoundError: No module named 'fastapi'`

**解决**:
```bash
pip install -r fingerprint_api/requirements.txt
```

### 问题 2: Redis 连接失败

**错误**: `redis.exceptions.ConnectionError: Error connecting to Redis`

**解决**:
```bash
# 检查 Redis 是否运行
redis-cli ping

# 启动 Redis
redis-server
```

### 问题 3: 所有请求都被限速 (429)

**原因**: 配额已耗尽或时钟偏差

**解决**:
```bash
# 重置用户配额
curl -X POST http://localhost:8000/api/v1/rate-limit/quota/{user_id}/reset

# 检查系统时间
date

# 检查 Redis 中的配额数据
redis-cli GET rl:quota:test_user_free
```

### 问题 4: 负载测试工具未安装

**k6 未安装**:
```bash
# Ubuntu
sudo apt-get install k6

# macOS
brew install k6

# Docker
docker run --network=host -v $(pwd)/tests/load:/scripts grafana/k6 run /scripts/k6_rate_limiting_test.js
```

**Apache Bench 未安装**:
```bash
# Ubuntu
sudo apt-get install apache2-utils

# macOS (已预装)
which ab
```

### 问题 5: 负载测试结果不符合预期

**Free 层应该 100-150 成功，但只有 50**:

1. 检查速率限制配置:
   ```python
   # fingerprint_api/config/rate_limit_config.py
   RATE_LIMIT_CONFIG = {
       "free": {"limit_per_minute": 100, "burst_multiplier": 1.5},
       ...
   }
   ```

2. 验证令牌桶逻辑:
   ```bash
   curl http://localhost:8000/api/v1/rate-limit/quota/test_user_free
   ```

3. 检查是否有并发限制:
   ```bash
   # 单用户单线程测试
   ab -n 150 -c 1 -H "X-API-Key: test_user" http://localhost:8000/api/v1/identify
   ```

---

## 性能调优

### 优化 1: Redis 连接池

编辑 `fingerprint_api/config/rate_limit_config.py`:

```python
REDIS_CONFIG = {
    "url": "redis://localhost:6379",
    "max_connections": 100,  # 增加连接池大小
    "connection_timeout": 5,
    "command_timeout": 2,
}
```

### 优化 2: 本地缓存

```python
CACHE_CONFIG = {
    "max_size": 10000,  # 增加缓存大小
    "ttl": 120,         # 增加 TTL (秒)
}
```

### 优化 3: Uvicorn Workers

```bash
# 增加 worker 数量 (通常 = CPU核心数)
uvicorn fingerprint_api.main:app --workers 8
```

---

## 下一步

完成负载测试后：

### 1. 查看测试报告

```bash
# 测试结果
ls -lh load_test_results/

# 生成汇总
bash scripts/generate_test_report.sh
```

### 2. 调整配置

根据测试结果调整速率限制配置：

```python
# fingerprint_api/config/rate_limit_config.py
TIER_CONFIGS = {
    QuotaTier.FREE: TierConfig(
        limit_per_minute=100,     # 根据需求调整
        monthly_quota=50000,
        burst_multiplier=1.5,
        cost_multipliers={...}
    ),
    ...
}
```

### 3. 部署到生产环境

参考：
- [PHASE_9_4_COMPLETE.md](./PHASE_9_4_COMPLETE.md) - 完整部署指南
- [PHASE_9_4_LOAD_TESTING_GUIDE.md](./PHASE_9_4_LOAD_TESTING_GUIDE.md) - 负载测试详解

### 4. 继续 Phase 9.5

Phase 9.5 将实施：
- 计费系统集成
- 使用追踪
- 超额计费
- 发票生成

---

## 资源链接

- **文档**:
  - [Phase 9.4 完整文档](./PHASE_9_4_COMPLETE.md)
  - [Kubernetes 基础设施](./PHASE_9_4_KUBERNETES_INFRASTRUCTURE.md)
  - [Rust 集成](./PHASE_9_4_RUST_INTEGRATION.md)
  - [Python 中间件](./PHASE_9_4_PYTHON_MIDDLEWARE_IMPLEMENTATION.md)
  - [负载测试指南](./PHASE_9_4_LOAD_TESTING_GUIDE.md)

- **代码**:
  - Python 中间件: `fingerprint_api/`
  - Rust 速率限制: `crates/fingerprint-core/src/rate_limiting*.rs`
  - 负载测试: `tests/load/`

- **工具**:
  - [k6 官网](https://k6.io/)
  - [FastAPI 文档](https://fastapi.tiangolo.com/)
  - [Redis 文档](https://redis.io/documentation)

---

## 故障排查

遇到问题？

1. 运行验证脚本:
   ```bash
   bash scripts/verify_load_test_ready.sh
   ```

2. 检查日志:
   ```bash
   # FastAPI 应用日志 (终端输出)
   # Redis 日志
   tail -f /var/log/redis/redis-server.log
   ```

3. 查看完整故障排查指南:
   - [PHASE_9_4_LOAD_TESTING_GUIDE.md](./PHASE_9_4_LOAD_TESTING_GUIDE.md) - "Troubleshooting"章节

---

**Phase 9.4 状态**: ✅ 生产就绪

祝测试愉快！🚀
