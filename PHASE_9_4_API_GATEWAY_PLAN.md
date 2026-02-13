# Phase 9.4: API Gateway & Distributed Rate Limiting

## 📋 Overview

**Status**: Planning Phase  
**Duration**: 30-40 hours  
**Expected Completion**: +4% progress (92% → 96%)  
**Priority**: High (Critical for production deployment)

---

## 🎯 Phase Objectives

### Primary Goals

1. **Deploy API Gateway** - Single entry point for all traffic
   - Request routing optimization
   - SSL/TLS termination
   - Request/response inspection
   - Path-based and host-based routing

2. **Implement Distributed Rate Limiting** - Redis-backed global limits
   - Per-user quotas
   - Per-IP rate limits
   - Per-endpoint throttling
   - Burst handling

3. **Establish User Quotas** - Billing-aware rate limiting
   - Free tier limits (100 req/min)
   - Pro tier limits (1000 req/min)
   - Enterprise unlimited (configurable)
   - Usage tracking for billing

4. **Enable Dynamic Policies** - Adaptive rate limiting
   - Automatic scaling based on load
   - Anomaly detection
   - Whitelist/blacklist management
   - Graceful degradation

---

## 📐 Architecture

```
┌──────────────────────────────────────┐
│        Internet / Users              │
└──────────────┬───────────────────────┘
               │
        (incoming requests)
               │
               ▼
    ┌──────────────────────┐
    │   API Gateway        │  (Kong / Nginx / Custom)
    ├──────────────────────┤
    │ • SSL/TLS termination│
    │ • Request routing    │
    │ • Load balancing     │
    └──────┬───────────────┘
           │
           ▼
    ┌──────────────────────┐
    │  Rate Limit Checker  │
    ├──────────────────────┤
    │ • User quota check   │
    │ • IP-based limits    │
    │ • Token bucket algo  │
    └──────┬───────────────┘
           │
           ▼
    ┌──────────────────────┐
    │  Redis Distributed   │  (Shared state)
    │  Rate Limit Store    │
    ├──────────────────────┤
    │ • User rate data     │
    │ • IP rate data       │
    │ • Quotas             │
    │ • Blacklists         │
    └──────────────────────┘
           │
    (Rate limit OK?)
           │
      ┌─────────┴──────────┐
      │ YES               NO
      ▼                   ▼
┌──────────────┐  ┌──────────────────────┐
│ Route to     │  │ Return 429           │
│ Backend      │  │ Rate Limited error   │
└────┬─────────┘  └──────────────────────┘
     │
     ▼
┌──────────────────────┐
│ fingerprint-api      │
│ • Processing        │
│ • Caching          │
│ • Response         │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Client Response      │
└──────────────────────┘
```

---

## 🔧 Technology Choices

### API Gateway Options

**Option A: Kong Gateway** ✅ Recommended
- Open-source, battle-tested
- Built-in rate limiting plugins
- Easy integration with Redis
- Admin API for dynamic config
- Plugin ecosystem mature
- **Pros**: Production-ready, feature-rich, easy to operate
- **Cons**: Memory footprint, operational complexity
- **Estimate**: 12 hours

**Option B: NGINX Ingress**
- Simpler than Kong
- NGINX AppProtect for rate limiting
- Native Kubernetes integration
- **Pros**: Lightweight, familiar
- **Cons**: Less feature-complete, more manual config
- **Estimate**: 10 hours

**Option C: Custom Rust Gateway**
- Full control, optimal performance
- Actix-web or Axum framework
- Direct Redis integration
- **Pros**: Minimal deps, tuned for fingerprint API
- **Cons**: More development work, operational burden
- **Estimate**: 25+ hours

**Recommendation**: Kong Gateway
- Provides 80/20 solution with minimal custom code
- Allows us to focus on business logic
- Proven production stability

### Rate Limiting Algorithm

**Token Bucket** (Recommended)
- Smooth traffic handling
- Burst allowance
- Fair distribution
- Standard implementation: Redis Sorted Sets + Lua scripts

**Implementation**:
```
Tokens per second = quota / time_period
Burst size = tokens_per_second * burst_multiplier (typically 1.5x)
Request cost = 1 token

Algorithm:
1. Check current tokens in bucket
2. If tokens >= cost: deduct cost, allow request
3. Otherwise: reject with 429 (Too Many Requests)
4. Refill bucket based on elapsed time
```

---

## 📦 Deliverables

### 1. Kong Gateway Deployment (12 hours)

**Files to Create**:
- `k8s/api-gateway/kong-deployment.yaml` (250+ lines)
  - Kong pod deployment
  - Service configuration
  - Storage for plugins
  - Resource limits

- `k8s/api-gateway/kong-postgres.yaml` (200+ lines)
  - PostgreSQL for Kong config
  - PersistentVolume
  - Database initialization

- `k8s/api-gateway/kong-plugins.yaml` (150+ lines)
  - Rate limiting plugin config
  - Authentication plugin
  - CORS plugin
  - Request/response transformation

- `k8s/api-gateway/kong-routes.yaml` (100+ lines)
  - API routes definition
  - Upstream service mapping
  - Plugin association

**Deployment Steps**:
1. Deploy PostgreSQL database
2. Deploy Kong pods
3. Initialize Kong admin API
4. Configure routes and plugins
5. Health check endpoints
6. Verify traffic routing

**Success Criteria**:
- Kong pods healthy and ready
- Admin API responding to requests
- Upstream fingerprint-api reachable
- No dropped requests during routing

### 2. Rate Limiting Middleware (14 hours)

**Files to Create**:
- `crates/fingerprint-core/src/rate_limiting.rs` (300+ lines)
  ```rust
  pub struct RateLimiter {
      redis: RedisClient,
      config: RateLimitConfig,
  }
  
  impl RateLimiter {
      pub async fn check_limit(&self, user_id: &str) -> Result<bool>
      pub async fn get_remaining(&self, user_id: &str) -> u32
      pub async fn check_bulk(&self, requests: Vec<RateRequest>) -> Vec<bool>
  }
  
  pub enum RateLimitConfig {
      PerUser { quota: u32, period: Duration },
      PerIp { quota: u32, period: Duration },
      Tiered { free: u32, pro: u32, enterprise: unlimited },
  }
  ```

- `phase7_api/middleware/rate_limit.py` (150+ lines)
  - FastAPI middleware
  - User extraction from headers
  - Response with rate limit headers
  - Graceful degradation

- `k8s/api-gateway/rate-limit-configmap.yaml` (100+ lines)
  - User tier configuration
  - Global rate limit settings
  - Refresh rates

**Implementation Details**:
- Token bucket with Lua scripts for atomicity
- Per-user quota from database
- IP-based fallback limits
- X-RateLimit-* response headers

### 3. User Quota Management (8 hours)

**Files to Create**:
- `crates/fingerprint-core/src/quota.rs` (200+ lines)
  ```rust
  pub struct QuotaManager {
      db: Arc<Database>,
      cache: Arc<Cache>,
  }
  
  pub struct UserQuota {
      user_id: String,
      tier: SubscriptionTier,
      quota_per_minute: u32,
      quota_per_month: u32,
      current_month_usage: u32,
  }
  ```

- `phase7_api/routes/quota.py` (100+ lines)
  - GET `/quota/info` - Current usage
  - GET `/quota/remaining` - Remaining quota
  - POST `/quota/upgrade` - Upgrade tier

**Quota Tiers**:
```
Free Tier:
- 100 requests / minute
- 10,000 requests / month
- $0/month

Pro Tier:
- 1,000 requests / minute
- 1,000,000 requests / month
- $99/month

Enterprise Tier:
- Custom limits
- 24/7 support
- Contact sales
```

### 4. Monitoring & Alerting (8 hours)

**Files to Create**:
- `monitoring/api-gateway-monitoring.yaml` (200+ lines)
  - Prometheus ServiceMonitor for Kong
  - PrometheusRule for rate limit alerts
  - Alert definitions:
    * High rate limit rejections (>10%)
    * Gateway errors (5xx > 1%)
    * Backend unavailable
    * Redis connection issues

- `monitoring/api-gateway-dashboards.yaml` (250+ lines)
  - Grafana dashboard "API Gateway Health"
  - Panels:
    * Request rate (total)
    * Rate limit rejection rate
    * Response latency distribution
    * Error rate by endpoint
    * Active connections
    * Backend health status

**Metrics**:
```
kong_requests_total
kong_request_duration_seconds
kong_upstream_target_health
rate_limit_rejections_total
user_quota_usage
```

### 5. Documentation (6 hours)

**Files to Create**:
- `PHASE_9_4_IMPLEMENTATION.md` (600+ lines)
  - Complete technical specification
  - Kong configuration details
  - Rate limiting algorithm explanation
  - Quota management flow
  - API endpoint documentation

- `PHASE_9_4_DEPLOYMENT_GUIDE.md` (400+ lines)
  - Step-by-step deployment
  - Configuration reference
  - Troubleshooting guide
  - Performance tuning tips
  - Common issues and solutions

### 6. Integration & Testing (2 hours)

- Integration with fingerprint-api deployment
- Load testing of rate limiting
- Failover scenario testing
- Documentation validation

---

## 📊 Success Metrics

### Functional Requirements
- ✅ No request loss during normal operation
- ✅ Rate limiting activates at configured thresholds
- ✅ 429 responses properly formatted
- ✅ Redis backend tracks state correctly
- ✅ User quotas enforced per billing tier

### Performance Targets
- Gateway P95 latency: <20ms
- Rate limit check latency: <2ms
- Redis operations: <1ms
- Cache hit rate: >95% for quota checks

### Reliability Targets
- Gateway availability: 99.9%
- Cache availability: 99.95% (Redis Sentinel)
- Zero dropped requests on rate limit rejection
- Graceful degradation if Redis unavailable

---

## 🚀 Deployment Timeline

### Week 1: Kong Setup (Days 1-2)
- Deploy Kong + PostgreSQL cluster
- Configure initial routes
- Admin API setup
- Testing

### Week 1: Rate Limiting (Days 3-4)
- Implement Redis token bucket
- Kong plugin configuration
- User quota database schema
- Testing with concurrent requests

### Week 2: Quotas & Monitoring (Days 5-6)
- Implement QuotaManager
- Dashboard creation
- Alerting rules
- Load testing

### Week 2: Documentation & Hardening (Days 7)
- Complete documentation
- Security review
- Performance optimization
- Deployment to production

---

## 🔐 Security Considerations

1. **Rate Limit Evasion Prevention**
   - IP spoofing detection
   - Header validation
   - Distributed attack detection
   - Whitelist management

2. **Backend Protection**
   - Circuit breaker patterns
   - Timeout management
   - Overload shedding
   - Graceful degradation

3. **Data Protection**
   - Rate limit data in Redis (encrypted)
   - User quota tracking (audit log)
   - API key validation
   - Token exchange security

---

## 📈 Expected Outcomes

### Capacity Improvement
- **Before**: 1,000 req/sec (limited by single backend)
- **After**: 5,000+ req/sec (with load balancing + caching)
- **Per-user**: Fair share with 100-1000 req/min limits

### Operational Benefits
1. Easier scaling (add Kong replicas)
2. Better visibility (Prometheus metrics)
3. User self-service quota tracking
4. Billing integration ready

### Business Benefits
1. Monetization ready (tiered quotas)
2. SLA compliance possible (rate limit guarantees)
3. API stability improved
4. Reduced backend load

---

## 🎯 Next Phase (9.5)

**Phase 9.5: Billing & Metering** (20-30 hours)
- Usage metering and aggregation
- Stripe/payment provider integration
- Invoice generation
- Quota reset scheduling
- Free trial management

**Phase 9.6: Advanced Security** (20 hours)
- API key management UI
- OAuth2 implementation
- IP whitelist/blacklist
- DDoS protection (WAF)
- Audit logging

**Phase 10: Polish & Release** (20 hours)
- Performance optimization
- Documentation finalization
- Security audit
- Load testing at scale
- Production deployment

---

## 📝 Immediate Next Steps

1. **Decide on Gateway Choice**
   - If Kong: Proceed with deployment
   - If NGINX: Adjust timeline and estimates
   - If Custom: Plan extended schedule

2. **Prepare Infrastructure**
   - Reserve 4 CPU, 4GB memory for Kong pods
   - Setup PostgreSQL cluster for Kong state
   - Verify Redis connectivity

3. **Plan Integration**
   - Coordinate with Phase 9.3 (caching layer)
   - Ensure fingerprint-api is cache-enabled
   - Plan traffic migration strategy

---

**Ready to proceed with Phase 9.4 deployment** ✅

Estimated completion: 2 days with focused effort  
Current progress: 92% (after Phase 9.3)  
Target: 96% (after Phase 9.4)

