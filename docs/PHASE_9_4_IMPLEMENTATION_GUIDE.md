# Phase 9.4: API Gateway & Rate Limiting Implementation Guide

## Overview

Phase 9.4 implements a distributed API gateway with advanced rate limiting. This document covers the implementation of Kong API Gateway, PostgreSQL backend, and Rust-based rate limiting service.

## Architecture

```
Client Requests
      ↓
[Kong API Gateway] (3 replicas, LoadBalancer)
      ↓
[Rate Limiter] (checks quota, Redis backend)
      ↓
[Fingerprint API] (Phase 8.5 service)
      ↓
[Cache Layer] (Phase 9.3, Redis)
```

## Components Deployed

### 1. Kong PostgreSQL Database
**File**: `k8s/api-gateway/kong-postgres.yaml`

- **Purpose**: Stores Kong configuration, routes, and services
- **Specifications**:
  - Image: `postgres:15-alpine`
  - Storage: 20Gi PersistentVolume
  - Health checks: `pg_isready` probes
  - Automated migrations via Job

**Kubernetes Resources**:
```yaml
Deployment:                 # Single pod
StatefulSet:               # (alternative)
PersistentVolumeClaim:     # 20Gi
Service:                   # port 5432
Secret:                    # postgres-secret (password)
ConfigMap:                 # init.sql (schema setup)
Job:                       # kong-migrations (bootstrap)
```

### 2. Kong API Gateway
**File**: `k8s/api-gateway/kong-deployment.yaml`

- **Purpose**: API gateway with authentication, routing, and plugin system
- **Specifications**:
  - Image: `kong:3.4-alpine`
  - Replicas: 3 (high availability)
  - Ports:
    - 8000: HTTP traffic
    - 8443: HTTPS traffic
    - 8001: Admin API
    - 8100: Status/metrics

**Kubernetes Resources**:
```yaml
Deployment:                # 3 replicas with anti-affinity
Services:
  - kong-gateway          # LoadBalancer (public traffic)
  - kong-admin            # ClusterIP (admin access)
  - kong-status           # ClusterIP (health checks + metrics)
PodDisruptionBudget:      # minAvailable=2
```

**Environment Configuration**:
```yaml
KONG_DATABASE: postgres
KONG_PG_HOST: kong-postgres
KONG_PG_DATABASE: kong
KONG_PROXY_ACCESS_LOG: /dev/stdout
KONG_ADMIN_ACCESS_LOG: /dev/stdout
```

### 3. Kong Plugins Configuration
**File**: `k8s/api-gateway/kong-plugins.yaml`

**Enabled Plugins**:

| Plugin | Purpose | Configuration |
|--------|---------|---------------|
| rate-limiting | Quota enforcement | Redis backend, configurable limits |
| key-auth | API key validation | Header: `x-api-key` or param: `apikey` |
| jwt | JWT token validation | HS256/384/512 algorithms |
| cors | Cross-origin headers | Allow all origins (configurable) |
| request-transformer | Header injection | Adds tracking headers |

**Routes Defined**:
```
/identify        → fingerprint-api:3000/identify
/compare         → fingerprint-api:3000/compare
/batch           → fingerprint-api:3000/batch
/health          → fingerprint-api:3000/health
```

### 4. Rate Limiting Configuration
**File**: `k8s/api-gateway/rate-limiting-configmap.yaml`

#### Quota Tiers

**Free Tier**:
```
- Requests per minute: 100
- Monthly quota: 50,000
- Cost: $0
- Use case: Development/evaluation
```

**Pro Tier**:
```
- Requests per minute: 1,000
- Monthly quota: 1,000,000
- Cost: $99/month
- Use case: Production startups
```

**Enterprise Tier**:
```
- Requests per minute: Unlimited
- Monthly quota: Unlimited
- Cost: Custom (contact sales)
- Use case: Mission-critical applications
```

**Partner Tier**:
```
- Requests per minute: Unlimited
- Monthly quota: Unlimited
- Cost: Free (special partnership)
- Use case: Integration partners
```

#### Per-Endpoint Cost Multipliers

| Endpoint | Cost | Effective Free Limit | Notes |
|----------|------|---------------------|-------|
| /identify | 1.0x | 100/min | Standard fingerprint |
| /compare | 2.0x | 50/min | More computationally expensive |
| /batch | 1.0x per item | 100/min | Bulk operations |
| /health | 0.0x | Unlimited | Health checks exempt |

#### Burst Handling

- **Token Bucket Algorithm**: Tokens refill at rate = monthly_limit / 30 / 1440 (tokens per minute)
- **Burst Multiplier**: 1.5x - allows up to 150% of minute limit in short bursts
- **Window**: 60 seconds
- **Reset**: Sliding window (not fixed intervals)

#### Unauthenticated (IP) Limits

| Category | Limit | Purpose |
|----------|-------|---------|
| Default IPs | 30 req/min | Basic protection against abuse |
| Whitelist (10.0.0.0/8) | 1000 req/min | Internal services |
| Blacklist | Manual | Abuse blocking |

#### Response Headers

```
X-RateLimit-Limit: 1000          # Requests allowed in window
X-RateLimit-Remaining: 987       # Requests remaining
X-RateLimit-Reset: 1699564800    # Unix timestamp of reset
X-RateLimit-Tier: pro            # User's quota tier
X-Quota-Monthly-Remaining: 987345 # Monthly quota remaining
```

When rejected:
```
Retry-After: 60                   # Seconds until retry
X-RateLimit-Reset: 1699564800    # Next window reset
```

#### Graceful Degradation Policy

| Load Level | Behavior | Action |
|-----------|----------|--------|
| < 70% | Normal | Accept all valid requests |
| 70-85% | Elevated | Increase logging, start monitoring |
| 85-95% | High | Reject new connections at 10% rate |
| > 95% | Critical | Reject new connections at 50% rate, circuit breaker active |

### 5. Monitoring & Alerting
**File**: `monitoring/api-gateway-monitoring.yaml`

**Prometheus Metrics Collected**:

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| kong_proxy_requests_total | Total requests processed | N/A |
| kong_plugins_rate_limiting_requests_rejected | Rate-limited requests | > 100/sec |
| kong_upstream_target_requests_total | Upstream requests | N/A |
| kong_upstream_target_health | Upstream availability | == 0 |
| kong_db_reachable | Database connectivity | == 0 |
| kong_proxy_requests_duration_seconds | Latency histogram | P95 > 500ms |

**Alert Rules**:

| Alert | Severity | Condition | Action |
|-------|----------|-----------|--------|
| KongDown | Critical | Kong endpoints unreachable | Page on-call engineer |
| HighErrorRate | Warning | Error rate > 5% | Monitor for 5 min |
| HighRateLimitRejections | Warning | Rejections > 100/sec | Check quotas |
| KongUpstreamUnavailable | Critical | Backend service down | Failover/remediate |
| KongDatabaseDown | Critical | PostgreSQL down | Immediate outage |
| RateLimitingRedisDown | Warning | Redis unavailable | Monitoring only, failover to local |

**Grafana Dashboards**:

1. **Kong API Gateway Health & Performance**
   - Request rate (graph)
   - Error rate percentage (gauge)
   - Latency P95 (timeseries)
   - Upstream health status (stat)
   - Database connectivity (stat)

2. **Rate Limiter & Quota Management**
   - Requests by tier (pie chart)
   - Quota usage percentage (stacked area)
   - Cache hit ratio (gauge)
   - Rejections by reason (bar chart)

## Implementation Files Created

### Phase 9.4 Kubernetes Manifests

1. **kong-postgres.yaml** (383 lines)
   - PostgreSQL 15 setup with persistent storage
   - Initialization scripts and migrations
   - Health checks and resource limits

2. **kong-deployment.yaml** (342 lines)
   - Kong gateway deployment (3 replicas)
   - Services for gateway, admin, status
   - Security context and resource management

3. **kong-plugins.yaml** (224 lines)
   - Plugin configurations for all 5 enabled plugins
   - Service/Route definitions
   - Authentication setup

4. **rate-limiting-configmap.yaml** (331 lines)
   - Complete quota tier system
   - Endpoint cost definitions
   - Rate limit thresholds

5. **api-gateway-monitoring.yaml** (Prometheus/Grafana config)
   - ServiceMonitor definitions
   - PrometheusRule alert rules
   - Grafana dashboard definitions

### Rust Implementation

**File**: `crates/fingerprint-core/src/rate_limiting.rs` (400+ lines)

**Key Components**:

```rust
// Quota tier enumeration
pub enum QuotaTier {
    Free,           // 100 req/min, 50K/month
    Pro,            // 1000 req/min, 1M/month
    Enterprise,     // Unlimited
    Partner,        // Unlimited
}

// User quota state
pub struct UserQuota {
    pub user_id: String,
    pub tier: QuotaTier,
    pub available_tokens: f64,      // Token bucket
    pub last_refill: u64,           // Last refill timestamp
    pub month_requests: u64,         // Monthly counter
    pub month_start: u64,           // Monthly reset epoch
    pub total_requests: u64,        // All-time counter
    pub last_request: u64,          // Last activity
}

// Rate limiter service
pub struct RateLimiter {
    user_quotas: DashMap<String, UserQuota>,    // Distributed cache
    ip_quotas: DashMap<String, UserQuota>,      // IP fallback
    endpoints: Arc<RwLock<Vec<EndpointConfig>>>,
    metrics: Arc<RateLimiterMetrics>,
    redis_url: String,              // Redis backend
}

// Main API
impl RateLimiter {
    pub fn new(redis_url: String) -> Self { ... }
    
    pub fn check_limit(
        &self,
        user_id: Option<&str>,
        tier: QuotaTier,
        endpoint: &str,
        client_ip: Option<&str>,
    ) -> Result<RateLimitResponse, RateLimitError> { ... }
    
    pub fn cleanup_stale_entries(&self, retention_seconds: u64) { ... }
    
    pub fn metrics_snapshot(&self) -> MetricsSnapshot { ... }
}
```

**Features**:
- Token bucket algorithm with burst support (1.5x)
- Redis-backed distributed state
- In-process cache with DashMap (concurrent HashMap)
- Automatic stale entry cleanup
- Comprehensive metrics (rejection rate, cache hit rate)
- Multi-tier quota system
- IP-based fallback for unauthenticated requests

### Deployment Script

**File**: `scripts/deploy-phase-9-4.sh` (250+ lines)

**Deployment Steps**:

1. ✅ Pre-deployment checks
   - Kubernetes cluster accessibility
   - Monitoring namespace existing
   - Redis cluster availability

2. ✅ Deploy PostgreSQL database
   - Create api-gateway namespace
   - Deploy StatefulSet with PVC
   - Wait for pod ready (30s)
   - Run migrations job (wait for completion)

3. ✅ Deploy Kong gateway
   - Deploy 3 replicas with anti-affinity
   - Create services (gateway, admin, status)
   - Verify rollout (120s timeout)

4. ✅ Configure plugins & routes
   - Apply plugin definitions
   - Configure rate limiting (Redis backend)
   - Set up routing to fingerprint-api

5. ✅ Deploy rate limiting config
   - Create ConfigMap with quota tiers
   - Validate quota definitions

6. ✅ Deploy monitoring
   - Apply ServiceMonitor for Prometheus
   - Deploy PrometheusRule alert rules
   - Create Grafana dashboards

7. ✅ Verification & baseline
   - Health checks (Kong status)
   - Prometheus scraping validation
   - Rate limiting config verification
   - Baseline metricsfile creation

**Execution Time**: 10-15 minutes

**Rollback Procedure**:
```bash
# If deployment fails:
kubectl delete namespace api-gateway
# This removes all Phase 9.4 resources

# Or selective rollback:
kubectl rollout undo deployment/kong -n api-gateway
kubectl delete job/kong-migrations -n api-gateway
```

## Integration with Phase 8.5 & 9.3

### Connection Points

**Phase 8.5 (Fingerprint API)**:
```
Kong Routes → fingerprint-api service:3000
             (via Kubernetes DNS discovery)
```

**Phase 9.3 (Cache Layer)**:
```
Rate Limiter → Redis (redis-cluster.caching:6379)
             (for distributed quota state)

Kong Plugins → Redis (shared rate limit storage)
             (decision consistency across pods)
```

### Request Flow

```
1. Client calls: GET /identify?browser=chrome
2. Kong validates plugin chain:
   a. rate-limiting: Check quota (Redis)
   b. key-auth: Verify API key (if required)
   c. jwt: Validate token (if required)
   d. cors: Add headers
   e. request-transformer: Add tracking headers
3. If all pass: Route to fingerprint-api:3000
4. fingerprint-api caches results in Phase 9.3
5. Response sent to client with headers:
   - X-RateLimit-*
   - X-Quota-Tier
   - Cache-Control from Phase 9.3
```

## Testing Rate Limiting

### Load Test Commands

```bash
# 1. Get API key first
export API_KEY=$(kubectl get secret kong-api-key -n api-gateway -o jsonpath='{.data.key}' | base64 -d)

# 2. Test within quota (should succeed)
for i in {1..50}; do
  curl -X GET "http://kong-gateway.api-gateway/identify" \
       -H "x-api-key: $API_KEY"
done

# 3. Exceed quota (should get 429 responses)
for i in {1..200}; do
  curl -X GET "http://kong-gateway.api-gateway/identify" \
       -H "x-api-key: $API_KEY"
done

# 4. Check response headers
curl -v -X GET "http://kong-gateway.api-gateway/identify" \
     -H "x-api-key: $API_KEY"

# Should see:
# X-RateLimit-Remaining: 987
# X-RateLimit-Reset: 1699564800
# X-Quota-Tier: pro
```

### Verify Metrics

```bash
# 1. Port-forward to Prometheus
kubectl port-forward -n monitoring svc/prometheus 9090:9090

# 2. Access Prometheus UI
# http://localhost:9090

# 3. Query rate limiting metrics
# - rate(kong_plugins_rate_limiting_requests_rejected[5m])
# - rate(kong_proxy_requests_total[1m])
# - kong_upstream_target_health

# 4. Access Grafana dashboards
# http://localhost:3000 (via port-forward from monitoring)
```

## Performance Characteristics

### Latency Impact

- **Rate limiting check**: < 2ms (in-process cache + Redis)
- **Kong proxy overhead**: < 5ms (plugin chain)
- **Total Kong latency**: 5-10ms added to backend

### Throughput

- **Kong per-pod**: 2000+ req/sec (from Kong benchmarks)
- **3-pod cluster**: 6000+ req/sec theoretical
- **Rate limiting bottleneck**: Redis (pipeline commands)
- **Practical sustainable**: 4000-5000 req/sec with monitoring overhead

### Memory Usage

- **Kong pod**: 512Mi nominal, 1Gi limit
- **PostgreSQL**: 512Mi nominal, 1Gi limit
- **Rate limiter (Rust)**: < 100Mi (DashMap, small quotas)

## Operational Procedures

### Daily Maintenance

```bash
# 1. Check gateway health
kubectl get pods -n api-gateway
kubectl get svc -n api-gateway

# 2. Monitor quota usage
kubectl logs -n api-gateway -l app=kong | grep "rate-limiting"

# 3. Verify Redis connectivity
redis-cli -h redis-cluster.caching -p 6379 PING

# 4. Check alert status
# (In Prometheus)
```

### Quota Tier Adjustments

To modify quota limits:

1. Edit `rate-limiting-configmap.yaml`
2. Update relevant tier configuration
3. Run: `kubectl apply -f k8s/api-gateway/rate-limiting-configmap.yaml`
4. Changes apply to new connections immediately

### Scaling

Add Kong pod:
```bash
kubectl scale deployment kong -n api-gateway --replicas=4
```

## Next Phase (Phase 9.5)

**Billing & Metering Integration**:
- Per-user request tracking
- Stripe integration for billing
- Usage reports and invoicing
- Quota adjustment based on payment tier

**Expected timeline**: 20-30 hours after Phase 9.4 stabilization

## Troubleshooting

### Kong cannot connect to PostgreSQL
```bash
# Check password in secret
kubectl get secret kong-postgres-secret -n api-gateway -o yaml

# Check PostgreSQL is running
kubectl logs pod/kong-postgres-* -n api-gateway

# Test connection from Kong pod:
kubectl exec pod/kong-* -n api-gateway -- \
  psql -h kong-postgres -U kong -d kong -c "SELECT 1"
```

### Rate limiting not working
```bash
# Check Kong has rate limiting plugin loaded
kubectl exec pod/kong-* -n api-gateway -- \
  curl localhost:8001/plugins | grep rate-limiting

# Verify Redis connectivity
kubectl exec pod/kong-* -n api-gateway -- \
  redis-cli -h redis-cluster.caching PING
```

### High error rate
```bash
# Check Kong logs
kubectl logs -n api-gateway -l app=kong --tail=100

# Check resource usage
kubectl top pods -n api-gateway

# Check upstream service
kubectl get endpoints fingerprint-api -n default
```

## References

- Kong Documentation: https://docs.konghq.com
- Token Bucket Algorithm: https://en.wikipedia.org/wiki/Token_bucket
- Kubernetes Namespaces: https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces
- Prometheus Query Language: https://prometheus.io/docs/prometheus/latest/querying/basics
