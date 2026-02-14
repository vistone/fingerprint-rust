# Phase 9.4 Load Testing Guide

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



## Overview

This guide covers comprehensive load testing for the Phase 9.4 Rate Limiting system, including per-user rate limits, tier-based quotas, endpoint cost multipliers, and burst capacity.

## Testing Tools

### 1. k6 (Recommended for comprehensive testing)

**Installation**:
```bash
# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# macOS
brew install k6

# Docker
docker pull grafana/k6
```

**Features**:
- Realistic traffic simulation
- Custom metrics and thresholds
- Multiple concurrent virtual users
- WebSocket support
- Results export (JSON, CSV, InfluxDB)

### 2. Apache Bench (ab)

**Installation**:
```bash
# Ubuntu/Debian
sudo apt-get install apache2-utils

# macOS (pre-installed)
# Already available as `ab`
```

**Features**:
- Simple and fast
- Good for basic benchmarking
- Request timing distribution

### 3. wrk (Advanced HTTP benchmarking)

**Installation**:
```bash
# Ubuntu/Debian
sudo apt-get install wrk

# macOS
brew install wrk

# Build from source
git clone https://github.com/wg/wrk.git
cd wrk
make
```

---

## Test Scenarios

### Scenario 1: Free Tier Rate Limiting (100 req/min)

**Objective**: Verify Free tier users are limited to 100 requests per minute

**k6 Test**:
```bash
k6 run --vus 1 --duration 2m \
  --env API_URL=http://localhost:8000 \
  tests/load/k6_rate_limiting_test.js
```

**Apache Bench Test**:
```bash
# Send 150 requests rapidly
ab -n 150 -c 1 \
   -H "X-API-Key: test_user_free" \
   -H "X-Quota-Tier: free" \
   -p tests/load/payloads/identify.json \
   -T "application/json" \
   http://localhost:8000/identify
```

**Expected Results**:
- First ~100-150 requests succeed (200 OK) - burst capacity allows > 100
- Remaining requests rejected (429 Too Many Requests)
- X-RateLimit-Remaining header counts down
- X-RateLimit-Reset header indicates when limit resets

### Scenario 2: Pro Tier Rate Limiting (1000 req/min)

**Objective**: Verify Pro tier users get 10x higher limit

**k6 Test**:
```bash
k6 run --vus 10 --duration 2m \
  --env API_URL=http://localhost:8000 \
  --env USER_TIER=pro \
  tests/load/k6_rate_limiting_test.js
```

**Apache Bench Test**:
```bash
# Send 1200 requests with 10 concurrent connections
ab -n 1200 -c 10 \
   -H "X-API-Key: test_user_pro" \
   -H "X-Quota-Tier: pro" \
   -p tests/load/payloads/identify.json \
   -T "application/json" \
   http://localhost:8000/identify
```

**Expected Results**:
- First ~1000-1500 requests succeed (burst capacity)
- Pro tier handles 10x more traffic than Free
- Response times remain consistent

### Scenario 3: Enterprise/Partner Tier (Unlimited)

**Objective**: Verify unlimited tiers never hit rate limits

**k6 Test**:
```bash
k6 run --vus 50 --duration 5m \
  --env API_URL=http://localhost:8000 \
  --env USER_TIER=enterprise \
  tests/load/k6_rate_limiting_test.js
```

**Expected Results**:
- Zero 429 responses
- All requests succeed
- Consistent response times

### Scenario 4: Endpoint Cost Multipliers

**Objective**: Verify /compare endpoint consumes 2x tokens

**Test**:
```bash
# Compare endpoint (2x cost)
ab -n 120 -c 1 \
   -H "X-API-Key: test_user_cost" \
   -H "X-Quota-Tier: free" \
   -p tests/load/payloads/compare.json \
   -T "application/json" \
   http://localhost:8000/compare
```

**Expected Results**:
- ~50-75 successful requests (100 tokens / 2.0 cost)
- Cost multiplier correctly enforced
- Faster rate limit exhaustion than /identify

### Scenario 5: Concurrent Independent Users

**Objective**: Verify each user has independent quota

**Test**:
```bash
# Run 10 concurrent users, each making 120 requests
for i in {1..10}; do
  ab -n 120 -c 1 \
     -H "X-API-Key: concurrent_user_$i" \
     -H "X-Quota-Tier: free" \
     -p tests/load/payloads/identify.json \
     -T "application/json" \
     http://localhost:8000/identify &
done
wait
```

**Expected Results**:
- Each user gets ~100-150 successful requests
- Total ~1000-1500 successful (10 users × 100-150)
- Users don't interfere with each other

### Scenario 6: Token Bucket Refill

**Objective**: Verify tokens refill over time

**Test**:
```bash
# Sustained load at 120 req/min for 3 minutes
ab -t 180 -c 2 -n 999999 \
   -H "X-API-Key: test_user_refill" \
   -H "X-Quota-Tier: free" \
   -p tests/load/payloads/identify.json \
   -T "application/json" \
   http://localhost:8000/identify
```

**Expected Results**:
- Average ~100 requests/min succeed
- Token bucket refills every 60 seconds
- Sustained throughput matches tier limit

### Scenario 7: Burst Capacity (1.5x)

**Objective**: Verify burst capacity allows temporary spikes

**Test**:
```bash
# Send 150 requests as fast as possible
ab -n 150 -c 10 \
   -H "X-API-Key: test_user_burst" \
   -H "X-Quota-Tier: free" \
   -p tests/load/payloads/identify.json \
   -T "application/json" \
   http://localhost:8000/identify
```

**Expected Results**:
- First ~150 requests succeed (1.5 × 100 burst)
- Burst capacity enables traffic spikes
- After burst, falls back to base limit

### Scenario 8: IP-Based Fallback (Unauthenticated)

**Objective**: Verify IP-based rate limiting for requests without API key

**Test**:
```bash
# Send 50 requests without authentication
ab -n 50 -c 1 \
   -p tests/load/payloads/identify.json \
   -T "application/json" \
   http://localhost:8000/identify
```

**Expected Results**:
- First ~30-45 requests succeed (30/min IP limit with burst)
- X-Forwarded-For header respected
- IP-based fallback protects against unauthenticated abuse

### Scenario 9: Health Endpoint Exemption

**Objective**: Verify /health is never rate limited

**Test**:
```bash
# Send 1000 requests to health endpoint
ab -n 1000 -c 10 http://localhost:8000/health
```

**Expected Results**:
- All 1000 requests succeed
- Zero 429 responses
- Exempt endpoints don't consume tokens

### Scenario 10: Mixed Traffic Patterns

**Objective**: Simulate realistic production traffic

**k6 Test**:
```bash
k6 run --vus 100 --duration 10m \
  --env API_URL=http://localhost:8000 \
  tests/load/k6_rate_limiting_test.js
```

**Expected Results**:
- Mix of Free (70%), Pro (25%), Enterprise (5%) users
- Realistic request patterns with delays
- System stable under sustained load

---

## Running Load Tests

### Quick Start (Apache Bench)

```bash
# Make script executable
chmod +x tests/load/ab_rate_limiting_test.sh

# Run all tests
bash tests/load/ab_rate_limiting_test.sh

# Run with custom duration
TEST_DURATION=300 bash tests/load/ab_rate_limiting_test.sh

# Run against staging
API_URL=https://api.staging.example.com bash tests/load/ab_rate_limiting_test.sh
```

### Comprehensive Testing (k6)

```bash
# Basic test
k6 run tests/load/k6_rate_limiting_test.js

# High load test
k6 run --vus 100 --duration 10m tests/load/k6_rate_limiting_test.js

# Export results to JSON
k6 run --out json=results.json tests/load/k6_rate_limiting_test.js

# Send results to InfluxDB
k6 run --out influxdb=http://localhost:8086/k6 tests/load/k6_rate_limiting_test.js
```

### Advanced Testing (wrk)

```bash
# Benchmark with wrk
wrk -t4 -c100 -d30s \
  -H "X-API-Key: test_user" \
  -H "X-Quota-Tier: free" \
  http://localhost:8000/identify

# With Lua script for POST requests
wrk -t4 -c100 -d30s \
  -s tests/load/wrk_post.lua \
  http://localhost:8000/identify
```

---

## Analyzing Results

### Apache Bench Output

```
Concurrency Level:      1
Time taken for tests:   15.456 seconds
Complete requests:      150
Failed requests:        50
Non-2xx responses:      50
Total transferred:      45000 bytes
Requests per second:    9.71 [#/sec] (mean)
Time per request:       103.04 [ms] (mean)
```

**Key Metrics**:
- **Complete requests**: Total requests sent
- **Failed requests**: Network/connection failures
- **Non-2xx responses**: Rate limited (429) + errors
- **Requests per second**: Throughput
- **Time per request**: Average latency

### k6 Output

```
     ✓ status is 200 or 429
     ✓ has rate limit remaining header
     ✓ has rate limit reset header

     checks.........................: 100.00% ✓ 12000      ✗ 0
     data_received..................: 3.6 MB  120 kB/s
     http_req_duration..............: avg=15.2ms min=5.1ms max=102.3ms
     http_reqs......................: 4000    133.33/s
     rate_limit_errors..............: 1200    40/s
     successful_requests............: 2800    93.33/s
```

**Key Metrics**:
- **checks**: Custom assertions (should be 100%)
- **http_req_duration**: Response time distribution
- **rate_limit_errors**: 429 responses (expected)
- **successful_requests**: 200 responses

### Grafana Dashboard

View real-time metrics in Grafana:

```bash
# Open Grafana
http://localhost:3000/d/rate-limiting

# Panels to monitor:
1. Request Rate (req/sec)
2. Rejection Rate (%)
3. Response Time (P50, P95, P99)
4. Active Users by Tier
5. Quota Utilization
6. Cache Hit Ratio
```

### Prometheus Queries

```bash
# Fetch metrics
curl http://localhost:8000/api/v1/rate-limit/metrics

# Key metrics:
rate_limit_total_requests            # Total requests
rate_limit_rejected_total            # Rejected requests
rate_limit_rejection_ratio           # Rejection rate
cache_hits_total                     # Cache performance
rate_limit_active_users              # Active users
```

**Prometheus Queries**:

```promql
# Request rate (per second)
rate(rate_limit_total_requests[5m])

# Rejection rate (percentage)
rate_limit_rejection_ratio * 100

# Average response time
rate(http_req_duration_sum[5m]) / rate(http_req_duration_count[5m])

# P95 response time
histogram_quantile(0.95, rate(http_req_duration_bucket[5m]))
```

---

## Performance Benchmarks

### Target Performance

| Metric | Free Tier | Pro Tier | Enterprise |
|--------|-----------|----------|------------|
| **Throughput** | 100 req/min | 1,000 req/min | unlimited |
| **Burst Capacity** | 150 req | 1,500 req | N/A |
| **Monthly Quota** | 50,000 | 1,000,000 | unlimited |
| **Response Time (P95)** | <100ms | <100ms | <100ms |
| **Cache Hit Ratio** | >80% | >80% | >80% |

### System Capacity

| Metric | Value |
|--------|-------|
| **Concurrent Users** | 10,000+ |
| **Total Throughput** | 100,000+ req/min |
| **Redis Ops** | 100,000+ ops/sec |
| **Memory per User** | ~200 bytes |
| **CPU Usage** | <50% @ 10K users |

---

## Troubleshooting

### Issue 1: All Requests Rate Limited

**Symptoms**: 100% 429 responses

**Diagnosis**:
```bash
# Check rate limit service status
curl http://localhost:8000/api/v1/rate-limit/status

# Check user quota
curl http://localhost:8000/api/v1/rate-limit/quota/test_user
```

**Solutions**:
- Reset user quota: `curl -X POST http://localhost:8000/api/v1/rate-limit/quota/{user_id}/reset`
- Verify tier configuration
- Check for system clock skew

### Issue 2: No Rate Limiting Enforced

**Symptoms**: 0% 429 responses, unlimited requests

**Diagnosis**:
```bash
# Check middleware is registered
curl -I http://localhost:8000/identify | grep X-RateLimit-Remaining

# Check service logs
docker logs fingerprint-api | grep -i rate
```

**Solutions**:
- Verify RateLimitMiddleware is registered
- Check RATE_LIMIT_ENABLED=true in config
- Restart API service

### Issue 3: High Response Times

**Symptoms**: P95 > 500ms

**Diagnosis**:
```bash
# Check Redis latency
redis-cli --latency -h redis-sentinel

# Check Prometheus metrics
curl http://localhost:8000/api/v1/rate-limit/metrics | grep duration
```

**Solutions**:
- Increase Redis connection pool
- Enable Redis pipelining
- Scale out Redis with Sentinel
- Tune Kong worker processes

### Issue 4: Low Cache Hit Ratio

**Symptoms**: cache_hit_ratio < 50%

**Diagnosis**:
```bash
# Check cache metrics
curl http://localhost:8000/api/v1/rate-limit/metrics/json | jq '.metrics.cache_hit_ratio'
```

**Solutions**:
- Increase CACHE_MAX_SIZE
- Increase CACHE_TTL_SECONDS
- Review cache eviction policy

---

## Load Test Checklist

### Pre-Test

- [ ] API service running and healthy
- [ ] Redis backend accessible
- [ ] Prometheus scraping metrics
- [ ] Grafana dashboard configured
- [ ] Test payloads prepared
- [ ] Baseline metrics recorded

### During Test

- [ ] Monitor Grafana dashboard
- [ ] Watch for error spikes
- [ ] Check Redis memory usage
- [ ] Verify Kong gateway health
- [ ] Monitor system resources (CPU, memory)

### Post-Test

- [ ] Analyze results (success/failure ratio)
- [ ] Review response time distribution
- [ ] Check for errors in logs
- [ ] Verify rate limiting accuracy
- [ ] Document findings
- [ ] Optimize based on results

---

## Continuous Load Testing

### CI/CD Integration

```yaml
# .github/workflows/load-test.yml
name: Load Test

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install k6
        run: |
          sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6
      
      - name: Run load test
        run: k6 run --vus 50 --duration 5m tests/load/k6_rate_limiting_test.js
      
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: load-test-results
          path: summary.json
```

### Performance Regression Detection

```bash
# Compare results with baseline
python scripts/compare_load_test_results.py \
  --baseline baseline_results.json \
  --current summary.json \
  --threshold 10  # 10% regression tolerance
```

---

## Next Steps

After completing load testing:

1. **Optimize Performance**
   - Tune Redis connection pool
   - Enable caching optimizations
   - Review bottlenecks in Prometheus

2. **Scale Infrastructure**
   - Add Kong replicas if needed
   - Scale Redis with Sentinel
   - Increase resource limits

3. **Production Deployment**
   - Deploy to staging first
   - Run production load tests
   - Monitor closely for 24-48 hours

4. **Phase 9.5: Billing Integration**
   - Connect rate limiting to billing
   - Track usage per endpoint
   - Generate invoices based on quota

---

## Summary

✅ **Load Testing Complete**

- Multiple test scenarios covering all rate limiting features
- Tools: k6 (comprehensive), Apache Bench (quick), wrk (advanced)
- Automated test scripts for CI/CD
- Grafana dashboards for visualization
- Performance benchmarks documented

**System Ready For**: Production deployment after load test validation
