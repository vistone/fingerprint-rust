# Phase 9.3: Advanced Caching Strategies - Deployment Guide

**Start Date**: 2026-02-13  
**Status**: 🔄 Deploying  
**Components**: 8 files, 2,500+ lines  

---

## 快速开始

### 前置条件

- Kubernetes 1.25+ 集群
- Phase 8 monitoring 已部署
- Phase 9.1/9.2 基础设施已就位

### 自动部署（推荐）

```bash
# 使部署脚本可执行
chmod +x scripts/deploy-phase-9-3.sh

# 执行部署（所有 5 个步骤）
./scripts/deploy-phase-9-3.sh

# 预计时间: 10-15 分钟
```

### 验证部署

```bash
# 1. 检查 Redis 集群状态
kubectl get pods -n caching -w
# 应该看到 3 个 redis-0/1/2 Pod 运行中

# 2. 验证主从复制
kubectl exec -n caching redis-0 -- redis-cli info replication
# 应该显示 "connected_slaves:2"

# 3. 访问监控仪表板
kubectl port-forward -n monitoring svc/grafana 3000:3000
# 访问 http://localhost:3000
# 导入仪表板: "Cache Performance Analytics"

# 4. 测试缓存函数
kubectl port-forward -n caching svc/redis-cluster 6379:6379
redis-cli SET test:key "hello"
redis-cli GET test:key  # 应该返回 "hello"
```

---

## 部署步骤详解

### Step 1: Redis 集群部署

**目的**: 部署 3 节点 Redis，支持自动故障转移

**文件**: `k8s/caching/redis-statefulset.yaml`

**执行**:
```bash
kubectl apply -f k8s/caching/redis-statefulset.yaml
```

**验证**:
```bash
# 等待所有 3 个 Pod 就绪
kubectl wait --for=condition=Ready pod -l app=redis -n caching --timeout=600s

# 检查副本状态
kubectl exec -n caching redis-0 -- redis-cli info replication
# 输出应包括: connected_slaves:2

# 检查 Sentinel 状态
kubectl exec -n caching redis-0 -c sentinel -- redis-cli -p 26379 sentinel masters
# 应该列出 mymaster (redis-0.redis.caching.svc.cluster.local 6379)
```

**关键指标**:
- Master: redis-0，地址: redis-0.redis.caching.svc.cluster.local:6379
- Slave 1: redis-1，自动复制
- Slave 2: redis-2，自动复制
- Sentinel: 3 个 Sentinel 监控和自动转移

### Step 2: 服务和监控部署

**目的**: 暴露 Redis 服务，启用 Prometheus 监控

**文件**: `k8s/caching/redis-service.yaml` + 监控配置

**执行**:
```bash
kubectl apply -f k8s/caching/redis-service.yaml
kubectl apply -f monitoring/redis-monitoring.yaml
kubectl apply -f monitoring/cache-dashboards.yaml
```

**服务**:
```yaml
redis                   # Headless Service (StatefulSet)
redis-cluster           # ClusterIP (应用访问)
redis-monitor          # NodePort 30379 (调试)
```

**监控**:
- ServiceMonitor: `redis` (30s 抓取间隔)
- PrometheusRule: `redis-caching` (8 条告警规则)
- Grafana: 2 个仪表板

### Step 3: 缓存管理部署

**目的**: 部署缓存预热、失效监视器 CronJob

**文件**: `k8s/caching/cache-management.yaml`

**执行**:
```bash
kubectl apply -f k8s/caching/cache-management.yaml
```

**组件**:

1. **cache-warmer** CronJob (每天 02:00 UTC)
   - 模式: full (加载所有浏览器配置)
   - 超时: 30 分钟
   - 重试: 2 次

2. **cache-warmer-hot** CronJob (每 6 小时)
   - 模式: hot (Chrome 最新版本)
   - 超时: 10 分钟

3. **redis-backup** CronJob (每 6 小时)
   - 触发 BGSAVE
   - 备份 RDB 文件

4. **cache-invalidation-watcher** Deployment (2 副本)
   - 监听 Redis PubSub 通道
   - 同步 L1 本地缓存失效

### Step 4：应用集成

**目的**: 配置 fingerprint-api 使用缓存

**环境变量**:
```bash
CACHE_REDIS_ADDR=redis-cluster.caching:6379
CACHE_ENABLED=true
CACHE_L1_TTL_SECS=300        # 5 分钟
CACHE_L2_TTL_SECS=1800       # 30 分钟
CACHE_L1_MAX_SIZE=10000      # 条数
CACHE_L2_MAX_SIZE=100000     # 条数
```

**执行**:
```bash
./scripts/deploy-phase-9-3.sh  # 自动设置
# 或手动:
kubectl set env deployment/fingerprint-api \
  -n fingerprint-api \
  CACHE_REDIS_ADDR="redis-cluster.caching:6379" \
  CACHE_ENABLED="true" \
  -c fingerprint-api
```

---

## 故障排查

### Redis 集群不就绪

**症状**: Pod 停留在 Pending 或 CrashLoopBackOff

**排查**:
```bash
# 1. 检查事件
kubectl describe pod redis-0 -n caching

# 2. 查看日志
kubectl logs -n caching redis-0 -c redis
kubectl logs -n caching redis-0 -c sentinel

# 3. 检查存储
kubectl get pvc -n caching
# 应该显示 3 个 10Gi PVC

# 4. 检查节点资源
kubectl top nodes
# 确保至少 3 个节点各有 1Gi 内存可用
```

**常见原因**:
- 存储不足: `PersistentVolume` 已满
- 内存不足: 节点内存 <2Gi
- 网络问题: Pod 间通信失败

### Sentinel 未转移故障

**症状**: Master 宕机但副本未升级

**排查**:
```bash
# 1. 检查 Sentinel 状态
kubectl exec -n caching redis-0 -c sentinel -- \
  redis-cli -p 26379 sentinel masters

# 2. 查看 Sentinel 日志
kubectl logs -n caching redis-0 -c sentinel --tail=50

# 3. 手动测试转移
kubectl delete pod -n caching redis-0
# 应该看到新的 Master 被选举
```

### 缓存命中率低

**症状**: `cache_hit_rate` < 70%

**排查**:
```bash
# 1. 检查缓存预热是否成功
kubectl logs -n fingerprint-api -l app=cache-warmer --tail=50

# 2. 查看缓存统计
kubectl exec -n caching redis-0 -- redis-cli INFO keyspace
# 应该显示 > 10,000 keys

# 3. 分析查询模式
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# 查询: rate(cache_misses_total[5m])

# 4. 调整 TTL
# 如果 L1 命中率太低，增加 CACHE_L1_TTL_SECS
# 如果 L2 命中率太低，增加缓存预热频率
```

### Prometheus 未抓取 Redis 指标

**症状**: Grafana 仪表板无数据

**排查**:
```bash
# 1. 验证 ServiceMonitor
kubectl get servicemonitors -n caching
kubectl describe servicemonitor redis -n caching

# 2. 检查 Prometheus 目标
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# 访问 http://localhost:9090/targets
# 查找 "redis" 作业，状态应为 UP

# 3. 查看 Prometheus 日志
kubectl logs -n monitoring -l app.kubernetes.io/name=prometheus --tail=50

# 4. 测试指标可用性
kubectl port-forward -n caching svc/redis-monitor 6379:6379
# 连接并运行: redis-cli INFO stats
```

---

## 性能优化

### 优化 L1 缓存

**L1 配置** (应用内存缓存):
```
容量: 10,000 条 (~10 MB)
TTL: 5 分钟
目标命中率: > 50%
```

**优化策略**:
1. 增加容量（如果内存允许）: `CACHE_L1_MAX_SIZE=20000`
2. 延长 TTL (如果数据新鲜度宽松): `CACHE_L1_TTL_SECS=600`
3. 分析热数据，预热 L1: 修改 cache-warmer 配置

### 优化 L2 缓存

**L2 配置** (Redis):
```
容量: 100,000 条 (~1 GB)
TTL: 30 分钟
目标命中率: > 80%
```

**优化策略**:
1. 增加 Replicas: `kubectl scale -n caching statefulset redis --replicas=5`
2. 增加内存: 修改 redis-statefulset.yaml `memory: 4Gi`
3. 启用 Redis Cluster: 将 Sentinel 升级为 Redis Cluster

### 监控和告警

**关键指标**:

| 指标 | 目标 | 告警 |
|------|------|------|
| 缓存命中率 | >85% | <70% |
| L1 命中率 | >50% | <30% |
| L2 命中率 | >80% | <60% |
| P95 延迟 | <10ms | >50ms |
| Redis 内存 | <80% | >80% |
| LRU 驱逐率 | <1/sec | >10/sec |

**Grafana 查询**:
```
# 总缓存命中率
sum(rate(cache_hits_total[5m])) / (sum(rate(cache_hits_total[5m])) + sum(rate(cache_misses_total[5m])))

# L1 命中率
sum(rate(cache_l1_hits_total[5m])) / (sum(rate(cache_l1_hits_total[5m])) + sum(rate(cache_l1_misses_total[5m])))

# Redis 内存使用
redis_memory_used_bytes / redis_memory_max_bytes

# 缓存查询延迟
histogram_quantile(0.95, rate(cache_query_duration_ms_bucket[5m]))
```

---

## 成功标准

✅ **Phase 9.3 验收标准**:

- [ ] Redis 集群 3 节点全部 Ready
- [ ] Sentinel 监控正常，转移测试通过
- [ ] Prometheus 抓取 Redis 指标 (>10 metrics)
- [ ] Grafana 2 个仪表板正常显示数据
- [ ] 缓存预热 CronJob 成功运行
- [ ] 缓存命中率 > 80% (稳定 1 小时)
- [ ] 缓存查询延迟 < 20ms P95
- [ ] 应用无缓存相关错误

**Timeline**:
- 部署时间: 10-15 分钟
- 预热时间: 30 分钟
- 基线建立: 1-2 小时
- 完全优化: 4-6 小时

---

## 常见问题

**Q: Redis Sentinel 和 Redis Cluster 有什么区别？**

A:
- **Sentinel**: 监控单个 master-slave，不分片，支持自动转移
- **Cluster**: 提前分片数据，自动转移，但更复杂
- 本 Phase 使用 Sentinel，适合中等规模

**Q: 缓存预热会影响性能吗？**

A: 预热在 02:00 UTC（流量低谷）进行，不影响。如需调整，修改 CronJob schedule: `"0 2 * * *"`

**Q: 如何实现缓存版本管理？**

A: 使用版本前缀 (如 `fingerprint:v1:user:123`)。Schema 变更时，在代码中递增 `CACHE_VERSION`，旧缓存自动失效。

**Q: 能否在多地区部署 Redis？**

A: Phase 9.1 已支持多地区。可在各地区独立部署 Redis，Phase 9.4 将添加跨地区缓存同步。

---

## 下一步

### Phase 9.4: API 网关和分布式速率限制 (30 小时)
- 基于 Redis 的全局速率限制
- 用户级别限流
- 动态限流策略

### Phase 9.5: 成本优化 (20 小时)
- 冷数据分层
- 自动扩展
- 成本分析

### Phase 10: 生产就绪 (20 小时)
- SRE 工具
- 应急预案
- 团队培训

---

## 快速参考

```bash
# 部署
./scripts/deploy-phase-9-3.sh

# 监控
kubectl port-forward -n monitoring svc/grafana 3000:3000      # Grafana
kubectl port-forward -n caching svc/redis-cluster 6379:6379  # Redis CLI
kubectl port-forward -n monitoring svc/prometheus 9090:9090   # Prometheus

# 调试
kubectl logs -n caching redis-0 -c redis -f
kubectl logs -n fingerprint-api -f -l app=cache-warmer
kubectl top pods -n caching

# 清理（如需回滚）
kubectl delete -f k8s/caching/
kubectl delete -f monitoring/redis-monitoring.yaml
kubectl delete -f monitoring/cache-dashboards.yaml
```

---

**状态**: Phase 9.3 部署就绪 ✅  
**预期完成**: 89% → 92% (项目进度)  
**下一步**: 部署后 1-2 小时验证基线
