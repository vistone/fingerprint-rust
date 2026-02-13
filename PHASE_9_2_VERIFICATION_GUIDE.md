# Phase 9.2: Service Mesh Advanced Features - Verification & Testing Guide

## Overview

This guide provides step-by-step verification procedures for Phase 9.2 deployment components including Jaeger distributed tracing, Kiali service mesh visualization, canary deployments, rate limiting, and advanced monitoring.

## Prerequisites

- Kubernetes cluster with 3+ nodes
- Istio service mesh 1.15+
- Prometheus 2.30+
- Grafana 8.0+
- kubectl configured with cluster access

## 1. Jaeger Distributed Tracing Verification

### 1.1 Verify Deployment Status

```bash
# Check Jaeger pods
kubectl get pods -n tracing -l app=jaeger

# Expected output: 2 replicas running
# NAME                                READY   STATUS    RESTARTS   AGE
# jaeger-xxx-yyy                      1/1     Running   0          5m
# jaeger-xxx-zzz                      1/1     Running   0          5m
```

### 1.2 Verify Service Connectivity

```bash
# Check service endpoints
kubectl get endpoints -n tracing jaeger-collector
kubectl get endpoints -n tracing jaeger-ui

# Port-forward to UI
kubectl port-forward -n tracing svc/jaeger-ui 16686:16686

# Access Jaeger UI at http://localhost:16686
```

### 1.3 Verify Trace Collection

```bash
# Check Jaeger collector logs for incoming traces
kubectl logs -n tracing -l app=jaeger -c jaeger --tail=50

# Look for: "Received spans" or similar collector messages

# Verify trace storage
kubectl exec -n tracing $(kubectl get pod -n tracing -l app=jaeger -o jsonpath='{.items[0].metadata.name}') -- \
  curl -s http://localhost:16686/api/services

# Should return list of services sending traces
```

### 1.4 Generate Test Traces

```bash
# Create a test deployment that sends traces
kubectl create deployment test-tracer --image=nginxdemos/hello:latest -n fingerprint-api

# Check if traces appear in Jaeger UI
# 1. Go to http://localhost:16686
# 2. Select service "test-tracer"
# 3. Click "Find Traces"
# 4. Should see traces with timestamps

# Clean up
kubectl delete deployment test-tracer -n fingerprint-api
```

## 2. Kiali Service Mesh Visualization Verification

### 2.1 Verify Deployment Status

```bash
# Check Kiali pods
kubectl get pods -n kiali -l app=kiali

# Expected: 2 replicas running
```

### 2.2 Verify Service Connectivity

```bash
# Port-forward to Kiali UI
kubectl port-forward -n kiali svc/kiali 20001:20001

# Access Kiali UI at http://localhost:20001
```

### 2.3 Verify RBAC Configuration

```bash
# Check ServiceAccount
kubectl get sa kiali -n kiali

# Verify ClusterRole permissions
kubectl get clusterrole kiali -o jsonpath='{.rules}' | jq .

# Check ClusterRoleBinding
kubectl get clusterrolebinding kiali
```

### 2.4 Verify Integration with Prometheus & Jaeger

```bash
# In Kiali UI (http://localhost:20001):
# 1. Go to Graph view
# 2. Select namespace: fingerprint-api
# 3. Verify service mesh displayed
# 4. Click on a service, check "Traces" tab
# 5. Should show traces from Jaeger integration

# Check Kiali configuration
kubectl get cm kiali -n kiali -o jsonpath='{.data.kiali\.yaml}' | grep -E "prometheus|tracing|grafana"
```

### 2.5 Verify Metrics Collection

```bash
# Query Kiali metrics from Prometheus
kubectl port-forward -n monitoring svc/prometheus 9090:9090

# In Prometheus (http://localhost:9090):
# Query: rate(istio_requests_total[5m])
# Should show response metrics from services

# Check request success rate
# Query: sum(rate(istio_requests_total{response_code=~"2.."}[5m])) by (destination_workload)
```

## 3. Canary Deployment Verification

### 3.1 Verify VirtualService Configuration

```bash
# Check canary VirtualService
kubectl get vs fingerprint-api-canary -n fingerprint-api -o yaml

# Verify traffic splitting rules
# Should show stable and canary subsets
# Verify route matching for x-canary header
```

### 3.2 Verify DestinationRule Subsets

```bash
# Check DestinationRule
kubectl get dr fingerprint-api-canary -n fingerprint-api -o yaml

# Verify subsets defined:
# - stable (version: stable)
# - canary (version: canary)

# Verify traffic policy settings
# Connection pool, outlier detection, load balancer
```

### 3.3 Verify Rate Limiting EnvoyFilter

```bash
# Check rate limiting filter
kubectl get envoyfilter -n fingerprint-api | grep rate-limiting

# Describe the filter
kubectl get envoyfilter rate-limiting-local -n fingerprint-api -o yaml

# Verify filter settings:
# - max_tokens: 1000
# - tokens_per_fill: 1000
# - fill_interval: 1s
```

### 3.4 Test Canary Traffic Splitting (if Flagger installed)

```bash
# Check Flagger Canary CRD
kubectl get canaries -n fingerprint-api

# If Flagger is deployed:
kubectl get crd canaries.flagger.app > /dev/null && {
  
  # Monitor canary status
  kubectl get canary fingerprint-api -n fingerprint-api -w
  
  # Check canary events
  kubectl describe canary fingerprint-api -n fingerprint-api
  
  # Expected phases: Initializing → Waiting → Progressing → Succeeded/Failed
}
```

### 3.5 Manual Canary Testing

```bash
# Send requests with canary header to test splitting
for i in {1..10}; do
  kubectl exec -n fingerprint-api $(kubectl get pod -l app=fingerprint-api -o jsonpath='{.items[0].metadata.name}') -- \
    curl -H "x-canary: true" http://fingerprint-api:8000/identify
done

# Without header (stable traffic)
for i in {1..10}; do
  kubectl exec -n fingerprint-api $(kubectl get pod -l app=fingerprint-api -o jsonpath='{.items[0].metadata.name}') -- \
    curl http://fingerprint-api:8000/identify
done

# Query metrics to see split
# In Prometheus: rate(istio_requests_total{destination_workload_version=~"stable|canary"}[1m])
```

## 4. Rate Limiting Verification

### 4.1 Verify Local Rate Limiting

```bash
# Generate high traffic to trigger rate limiting
kubectl run -it --rm=true test-load --image=busybox:1.28 --restart=Never -- /bin/sh

# Inside container, run ab or similar tool
# ab -n 2000 -c 100 http://fingerprint-api:8000/identify

# Observe:
# - Requests should be rate-limited after 1000/sec
# - Response headers should show: x-local-rate-limit: true
# - x-ratelimit-remaining header should show remaining tokens
```

### 4.2 Verify Rate Limit Headers

```bash
# Send request and check headers
kubectl exec -n fingerprint-api $(kubectl get pod -l app=fingerprint-api -o jsonpath='{.items[0].metadata.name}') -- \
  curl -v http://fingerprint-api:8000/identify 2>&1 | grep -i "x-ratelimit\|x-local-rate-limit"

# Expected headers:
# x-local-rate-limit: true
# x-ratelimit-limit: 1000
# x-ratelimit-remaining: 999
# x-ratelimit-reset: 1
```

### 4.3 Monitor Rate Limiting Metrics

```bash
# Port-forward Prometheus
kubectl port-forward -n monitoring svc/prometheus 9090:9090

# Query rate limiting metrics:
# rate(envoy_http_local_rate_limit_http_filter_ratelimit_enforced[5m])

# Should show enforcement events under high load
```

## 5. Advanced Monitoring Verification

### 5.1 Verify PrometheusRule Deployment

```bash
# Check custom alert rules
kubectl get prometheusrule -n monitoring | grep "service-mesh"

# Describe the rule
kubectl get prometheusrule service-mesh-advanced -n monitoring -o yaml

# Verify rules loaded in Prometheus
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# Visit http://localhost:9090/alerts
# Check for rules: CanaryErrorRateHigh, RateLimitExceeded, CircuitBreakerTriggered, etc.
```

### 5.2 Verify ServiceMonitor Configuration

```bash
# Check ServiceMonitors created
kubectl get servicemonitor -n monitoring | grep -E "istio-mesh|jaeger|kiali|envoy"

# Verify scrape targets
# In Prometheus: http://localhost:9090/targets
# Should see targets for:
# - istio-mesh (fingerprint-api)
# - jaeger
# - kiali
# - envoy-proxy
```

### 5.3 Verify Grafana Dashboards

```bash
# Check dashboard ConfigMaps
kubectl get cm -n monitoring | grep dashboard

# Import to Grafana:
kubectl port-forward -n monitoring svc/grafana 3000:3000
# Login with default credentials (admin:admin)
# Go to Dashboards → Upload JSON
# Select service-mesh-dashboard.json
# Verify panels display correctly

# Panels should include:
# 1. Canary Traffic Split (graph)
# 2. Canary Error Rate vs Stable (graph)
# 3. Request Latency P95 (Canary vs Stable)
# 4. Rate Limit Enforcement Count (stat)
# 5. Circuit Breaker Status (graph)
# 6. Jaeger Trace Throughput (graph)
# 7. Kiali Service Mesh Summary (stat)
# 8. Multi-Region Request Distribution (pie chart)
```

## 6. End-to-End Testing

### 6.1 Distributed Tracing End-to-End

```bash
# 1. Send an API request
kubectl exec -n fingerprint-api $(kubectl get pod -l app=fingerprint-api -o jsonpath='{.items[0].metadata.name}') -- \
  curl -H "x-trace-id: test-trace-123" http://fingerprint-api:8000/identify

# 2. Go to Jaeger UI (localhost:16686)
# 3. Search for trace: test-trace-123
# 4. Verify full trace depth shows all service calls
# 5. Check latency breakdown per service
```

### 6.2 Service Mesh Observability

```bash
# 1. Generate mix of traffic (canary + stable)
# 2. Go to Kiali UI (localhost:20001)
# 3. View Graph section
# 4. Should see:
#    - Service dependencies with traffic flow
#    - Canary deployment (if running)
#    - Error rates and success rates
#    - Latency information

# 5. Click on service → Traces tab
# 6. Should link to Jaeger traces
```

### 6.3 Monitoring & Alerting

```bash
# 1. Trigger rate limiting
ab -n 5000 -c 50 http://fingerprint-api.fingerprint-api.svc.cluster.local:8000/identify

# 2. Check Prometheus alerts
# Visit http://localhost:9090/alerts
# Verify "RateLimitExceeded" alert fires

# 3. Check Grafana notifications (if configured)
# Dashboard should show increased rate limit enforcement count

# 4. Test canary error rate high alert
# Modify canary to return 5xx errors
# After 2 minutes, alert should fire
```

## 7. Performance Baselines

### 7.1 Baseline Metrics

Record these metrics for performance comparison:

- **Canary Error Rate**: <0.1% (normal operation)
- **P95 Latency**: <500ms for canary, <300ms for stable
- **Rate Limit Enforcement**: <100 rejections/5min per pod
- **Trace Sampling**: 100% (adjustable in telemetry config)
- **Circuit Breaker Trips**: 0 (normal operation)

### 7.2 Test Scenario: Load Increase

```bash
# Generate increasing load and observe metrics
# Phase 1: 100 req/sec
# Phase 2: 500 req/sec
# Phase 3: 1000 req/sec
# Phase 4: 2000 req/sec (exceeds rate limit)

# At each phase, record:
# - Error rate
# - P95 latency
# - Rate limit enforcements
# - Circuit breaker status
```

## 8. Troubleshooting

### Issue: Jaeger traces not appearing

```bash
# 1. Check Jaeger collector logs
kubectl logs -n tracing -l app=jaeger -c jaeger --tail=100

# 2. Verify telemetry configuration
kubectl get telemetries -n fingerprint-api

# 3. Check if applications sending traces
kubectl logs -n fingerprint-api $(kubectl get pod -l app=fingerprint-api -o jsonpath='{.items[0].metadata.name}') | grep -i "jaeger\|trace"

# 4. Verify collector service DNS
kubectl run -it --image=busybox:1.28 --rm --restart=Never -- \
  wget -O- http://jaeger-collector.tracing:14268
```

### Issue: Kiali showing "No mesh configured"

```bash
# 1. Check Kiali RBAC
kubectl auth can-i get services --as=system:serviceaccount:kiali:kiali -n fingerprint-api

# 2. Verify namespace label
kubectl get ns fingerprint-api --show-labels
# Should have: istio-injection=enabled

# 3. Restart Kiali
kubectl rollout restart deployment/kiali -n kiali
```

### Issue: Rate limiting not working

```bash
# 1. Check EnvoyFilter
kubectl get envoyfilter -n fingerprint-api -o yaml | grep -A20 "local_ratelimit"

# 2. Start container shell and check Envoy config
kubectl exec -n fingerprint-api $(kubectl get pod -l app=fingerprint-api -o jsonpath='{.items[0].metadata.name}') -c fingerprint-api -- sh

# Inside: curl localhost:15000/config_dump | jq . | grep -i "rate.*limit"

# 3. Check sidecar proxy logs
kubectl logs -n fingerprint-api $(kubectl get pod -l app=fingerprint-api -o jsonpath='{.items[0].metadata.name}') -c istio-proxy --tail=50
```

## 9. Cleanup & Rollback

### Rollback Phase 9.2

```bash
# Remove advanced monitoring
kubectl delete -f monitoring/prometheus-rules-advanced.yaml
kubectl delete -f monitoring/servicemonitor.yaml
kubectl delete -f monitoring/grafana-dashboards-advanced.yaml

# Remove canary configs
kubectl delete -f k8s/networking/canary/flagger-canary.yaml
kubectl delete -f k8s/networking/canary/rate-limiting.yaml
kubectl delete -f k8s/networking/canary/virtualservice.yaml

# Remove telemetry
kubectl delete -f k8s/networking/istio/telemetry-config.yaml

# Remove Kiali
kubectl delete -f monitoring/kiali/kiali-deployment.yaml

# Remove Jaeger
kubectl delete -f monitoring/jaeger/jaeger-deployment.yaml
```

## 10. Success Criteria

✅ Phase 9.2 is considered complete when:

- [ ] Jaeger collecting traces from all services
- [ ] Kiali displaying service mesh graph
- [ ] Canary traffic splitting working (95/5 or manual split)
- [ ] Rate limiting enforced (1000 req/sec per pod)
- [ ] Prometheus rules evaluating correctly
- [ ] Grafana dashboards displaying real-time metrics
- [ ] All alerts triggering appropriately
- [ ] End-to-end tracing visible in Jaeger
- [ ] Service dependencies visible in Kiali
- [ ] No errors in operator logs

Target Coverage: 95% of Phase 9.2 advanced features operational
