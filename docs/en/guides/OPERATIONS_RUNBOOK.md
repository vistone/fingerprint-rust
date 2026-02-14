# Phase 8.5 Operations & Runbook Guide

**Phase**: 8.5 Production Operations Documentation  
**Status**: ✅ COMPLETE  
**Date**: 2026-02-13  
**Audience**: DevOps Engineers, SREs, Operations Teams  
**Revision**: 1.0  

---

## Table of Contents

1. [Daily Operations](#daily-operations)
2. [Incident Response Runbooks](#incident-response-runbooks)
3. [Scaling & Capacity Management](#scaling--capacity-management)
4. [Backup & Disaster Recovery](#backup--disaster-recovery)
5. [Performance Tuning](#performance-tuning)
6. [Health Checks & Diagnostics](#health-checks--diagnostics)
7. [Monitoring & Alerting](#monitoring--alerting)
8. [Common Issues & Solutions](#common-issues--solutions)
9. [Operations Checklist](#operations-checklist)
10. [Emergency Contacts & Escalation](#emergency-contacts--escalation)

---

## Daily Operations

### 1.1 Morning Checklist (Start of Shift)

```bash
#!/bin/bash
# Daily operations check

echo "=== Fingerprint API Morning Checklist ===" 
echo "[$(date)]"

# Check pod health
echo -e "\n1. Checking pod health..."
kubectl get pods -n fingerprint -o wide
READY_PODS=$(kubectl get pods -n fingerprint -o jsonpath='{.items[?(@.status.conditions[].status=="True")].metadata.name}' | wc -w)
echo "Ready pods: $READY_PODS/3"

# Check deployment rollout status
echo -e "\n2. Checking deployment status..."
kubectl rollout status deployment/fingerprint-api -n fingerprint --timeout=30s || echo "⚠️ Deployment not ready"

# Check service endpoints
echo -e "\n3. Checking service endpoints..."
kubectl get endpoints -n fingerprint fingerprint-api

# Check for recent pod restarts
echo -e "\n4. Checking pod restarts..."
kubectl get pods -n fingerprint -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.containerStatuses[0].restartCount}{"\n"}{end}'

# Check Prometheus alert status
echo -e "\n5. Checking Prometheus alerts..."
kubectl exec -n prometheus -it $(kubectl get pods -n prometheus -l app=prometheus -o jsonpath='{.items[0].metadata.name}') -- \
  curl -s http://localhost:9090/api/v1/alerts | jq '.data.alerts | length'

# Check PVC status (if using persistent volumes)
echo -e "\n6. Checking persistent volumes..."
kubectl get pvc -n fingerprint

# Check resource utilization
echo -e "\n7. Current resource utilization..."
kubectl top pods -n fingerprint

echo -e "\n=== Morning Checklist Complete ==="
```

**Expected Results**:
- ✅ 3+ pods in Running state
- ✅ Deployment rollout status: "deployment 'fingerprint-api' successfully rolled out"
- ✅ Service has active endpoints
- ✅ No unexpected restart counts (> 3)
- ✅ CPU/Memory usage within expected ranges

### 1.2 Hourly Status Check

```bash
#!/bin/bash
# Hourly monitoring check script

NAMESPACE="fingerprint"
THRESHOLD_ERROR_RATE=5  # 5% error rate threshold
THRESHOLD_LATENCY=1000  # 1000ms latency threshold

# Check error rate from Prometheus
echo "Checking error rate..."
ERROR_RATE=$(kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query?query=rate(http_requests_total{status=~"5.."}[5m])' | \
  jq '.data.result[0].value[1]' 2>/dev/null || echo "0")

if (( $(echo "$ERROR_RATE > $THRESHOLD_ERROR_RATE" | bc -l) )); then
  echo "⚠️ High error rate detected: ${ERROR_RATE}%"
fi

# Check latency
echo "Checking P99 latency..."
LATENCY=$(kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query?query=histogram_quantile(0.99,http_request_duration_seconds_bucket)' | \
  jq '.data.result[0].value[1]' 2>/dev/null || echo "0")

if (( $(echo "$LATENCY > $THRESHOLD_LATENCY" | bc -l) )); then
  echo "⚠️ High latency detected: ${LATENCY}ms"
fi

echo "✅ Hourly check complete"
```

### 1.3 Evening Handoff (End of Shift)

- Document any issues encountered in ticket system
- Update runbook with new learnings
- Verify all alerts have been acknowledged
- Confirm backup completion
- Handoff notes to next shift

---

## Incident Response Runbooks

### 2.1 Critical: API Service Down

**Symptom**: Pods not running or service unreachable  
**Severity**: P1 (Critical)  
**Response Time**: Immediate

#### Investigation Steps

```bash
# 1. Check pod status
kubectl get pods -n fingerprint -o wide

# 2. Describe failed pod
kubectl describe pod <pod-name> -n fingerprint

# 3. Check pod logs
kubectl logs <pod-name> -n fingerprint -c fingerprint-api --tail=100

# 4. Check previous pod logs (if crashed)
kubectl logs <pod-name> -n fingerprint -c fingerprint-api --previous

# 5. Check events
kubectl get events -n fingerprint --sort-by='.lastTimestamp'

# 6. Check service endpoints
kubectl get endpoints -n fingerprint fingerprint-api
```

#### Common Causes & Fixes

| Cause | Fix |
|-------|-----|
| Image pull error | `kubectl describe pod` - check image pull status |
| Resource limits | Increase resource requests/limits in deployment |
| Model loading failure | Check ConfigMap mount, verify model files exist |
| Disk full | `kubectl exec` → check `/tmp` space, cleanup if needed |
| DNS resolution | Verify Kubernetes DNS is working: `kubectl run -it --rm debug --image=busybox -- nslookup elasticsearch` |

#### Recovery Steps

```bash
# Option 1: Restart deployment
kubectl rollout restart deployment/fingerprint-api -n fingerprint

# Option 2: Scale to 0 then restore
kubectl scale deployment fingerprint-api -n fingerprint --replicas=0
sleep 10
kubectl scale deployment fingerprint-api -n fingerprint --replicas=3

# Option 3: Check and fix configuration
kubectl edit configmap fingerprint-config -n fingerprint
# Make corrections, save
kubectl rollout restart deployment/fingerprint-api -n fingerprint

# Monitor recovery
kubectl rollout status deployment/fingerprint-api -n fingerprint --timeout=5m
```

#### Escalation

- **5 minutes**: Notify on-call SRE if not resolved
- **15 minutes**: Escalate to DevOps lead
- **30 minutes**: Escalate to Platform engineering manager

---

### 2.2 Warning: High Error Rate (>5%)

**Symptom**: Prometheus alert "HighErrorRate" firing  
**Severity**: P2 (High)  
**Response Time**: 10 minutes

#### Investigation Steps

```bash
# 1. Check error rate and type
kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=sum(rate(http_requests_total{status=~"5.."}[5m])) by (status)'

# 2. Check error logs in Kibana
# Navigate to: http://kibana:5601
# Search: severity:ERROR AND timestamp:[now-30m TO now]

# 3. Check affected endpoints
kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=rate(http_requests_total{status=~"5.."}[5m]) by (endpoint)'

# 4. Check pod logs
kubectl logs -n fingerprint -l app=fingerprint-api --tail=200 | grep ERROR
```

#### Common Causes & Fixes

| Cause | Fix |
|-------|-----|
| Model loading failure | Check model files, verify feature dimensions match |
| Database connection issues | Verify database connectivity from pod |
| High load/timeout | Scale up with HPA or manual scaling |
| Invalid input data | Check input validation, maybe client sending malformed data |
| Memory pressure | Check memory limits, may need to increase |

#### Recovery Steps

```bash
# 1. If specific pod is erroring
kubectl delete pod <pod-name> -n fingerprint

# 2. If all pods affected, restart deployment
kubectl rollout restart deployment/fingerprint-api -n fingerprint

# 3. If error rate persists, scale down to get clean state
kubectl scale deployment fingerprint-api -n fingerprint --replicas=1
# Wait for pod to stabilize (30 seconds)
kubectl scale deployment fingerprint-api -n fingerprint --replicas=3

# 4. Monitor error rate decrease
watch 'kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s "http://localhost:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m])"'
```

---

### 2.3 Warning: High Latency (P99 > 1s)

**Symptom**: Response times exceed 1 second for 99th percentile  
**Severity**: P2 (High)  
**Response Time**: 15 minutes

#### Investigation Steps

```bash
# 1. Check latency percentiles
kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) by (endpoint)'

# 2. Check resource utilization
kubectl top pods -n fingerprint
kubectl top nodes

# 3. Check inference latency component
kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=histogram_quantile(0.95, rate(fingerprint_api_inference_duration_seconds_bucket[5m])) by (level)'

# 4. Check for queue buildup
kubectl logs -n fingerprint -l app=fingerprint-api --tail=100 | grep queue_depth
```

#### Common Causes & Fixes

| Cause | Fix |
|-------|-----|
| Resource contention | Scale up replicas, or check for pod evictions |
| High throughput | Increase workers/connections, scale horizontally |
| Model inference complexity | Profile with Prometheus metrics, consider model optimization |
| Garbage collection pauses | Adjust GC parameters in deployment env vars |
| Network latency | Check node networking, run network diagnostics |

#### Recovery Steps

```bash
# Option 1: Scale horizontally
kubectl scale deployment fingerprint-api -n fingerprint --replicas=5

# Option 2: Adjust request timeout
kubectl edit configmap fingerprint-config -n fingerprint
# Increase request_timeout value

# Option 3: Restart pods to trigger GC
kubectl rollout restart deployment/fingerprint-api -n fingerprint

# Monitor latency improvement
watch 'kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s "http://localhost:9090/api/v1/query?query=histogram_quantile(0.99,rate(http_request_duration_seconds_bucket[5m]))"'
```

---

### 2.4 Warning: Pod Crash Loop

**Symptom**: Pod restarting multiple times (>3 restarts in 10 minutes)  
**Severity**: P2 (High)  
**Response Time**: 10 minutes

#### Investigation Steps

```bash
# 1. Check pod restart count
kubectl get pods -n fingerprint -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.containerStatuses[0].restartCount}{"\n"}{end}'

# 2. Check crash logs
kubectl logs <pod-name> -n fingerprint --previous

# 3. Check last termination reason
kubectl describe pod <pod-name> -n fingerprint | grep -A 5 "Last State"

# 4. Check resource limits
kubectl get deployment fingerprint-api -n fingerprint -o yaml | grep -A 10 "resources:"
```

#### Common Causes & Fixes

| Cause | Fix |
|-------|-----|
| OOMKilled (out of memory) | Increase memory limits in deployment |
| CrashBackOff (app exit) | Check logs for application error |
| Liveness probe failing | Adjust probe timeout/threshold |
| Missing model files | Verify ConfigMap/volume mount |

#### Recovery Steps

```bash
# 1. Examine pod logs closely
kubectl logs <pod-name> -n fingerprint --previous --all-containers=true

# 2. If OOMKilled, increase memory
kubectl set resources deployment fingerprint-api \
  -n fingerprint \
  --limits=memory=3Gi \
  --requests=memory=1Gi

# 3. If application crash, check configuration
kubectl get configmap fingerprint-config -n fingerprint -o yaml

# 4. Restart deployment
kubectl rollout restart deployment/fingerprint-api -n fingerprint

# 5. Monitor for stability
kubectl get pods -n fingerprint -w
```

---

### 2.5 Critical: Disk Space Issues

**Symptom**: Logs filling up, Elasticsearch running out of disk space  
**Severity**: P1 (Critical)  
**Response Time**: Immediate

#### Investigation Steps

```bash
# 1. Check available disk space
kubectl exec -n fingerprint -it <pod-name> -- df -h

# 2. Find large directories
kubectl exec -n fingerprint -it <pod-name> -- du -sh /* | sort -rh

# 3. Check log disk usage
kubectl exec -n fingerprint -it <pod-name> -- du -sh /var/log

# 4. Check Elasticsearch disk
kubectl exec -n logging -it elasticsearch-0 -- curl -s localhost:9200/_nodes/stats/fs | jq '.nodes[].fs'
```

#### Recovery Steps

```bash
# Option 1: Clean old logs Locally
kubectl exec -n fingerprint -it <pod-name> -- sh -c 'find /var/log -name "*.log" -mtime +7 -delete'

# Option 2: Trigger log rotation
kubectl exec -n fingerprint -it <pod-name> -- logrotate -f /etc/logrotate.d/fingerprint

# Option 3: For Elasticsearch, delete old indices
kubectl exec -n logging -it elasticsearch-0 -- \
  curl -X DELETE localhost:9200/fingerprint-api-2026.01.* -H 'Content-Type: application/json'

# Option 4: Add more disk space (if using PVC)
# Edit PVC size in k8s/base or contact infrastructure team

# 5. Verify space recovered
kubectl exec -n fingerprint -it <pod-name> -- df -h
```

---

## Scaling & Capacity Management

### 3.1 Manual Scaling

#### Scale Replicas

```bash
# Scale to specific number
kubectl scale deployment fingerprint-api -n fingerprint --replicas=5

# Get HPA to pause auto-scaling (if needed)
kubectl describe hpa fingerprint-api-hpa -n fingerprint

# Manual override (within HPA limits)
# Clear HPA by patching deployment replicas
kubectl patch deployment fingerprint-api -n fingerprint \
  -p '{"spec":{"replicas":5}}'
```

#### Verify Scaling

```bash
# Watch scaling progress
kubectl get pods -n fingerprint -w

# Check rollout status
kubectl rollout status deployment/fingerprint-api -n fingerprint

# Verify load distribution
kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=count(kube_pod_status_phase{phase="Running"})'
```

### 3.2 Resource Limit Adjustments

```bash
# Edit resource requests/limits
kubectl set resources deployment fingerprint-api -n fingerprint \
  --requests=cpu=750m,memory=768Mi \
  --limits=cpu=2000m,memory=2Gi

# Verify changes applied
kubectl get deployment fingerprint-api -n fingerprint -o yaml | grep -A 8 resources:

# Monitor pod restart during update
kubectl get pods -n fingerprint -w
```

### 3.3 Capacity Planning

**Current Metrics**:
- Base: 3 replicas × 512Mi = 1.5Gi memory
- Typical load: 5 replicas × 512Mi = 2.5Gi memory
- Peak capacity: 10 replicas × 2Gi = 20Gi memory

**Planning Guidelines**:

| Load Level | Replicas | Memory Required | CPU Required |
|-----------|----------|-----------------|--------------|
| Baseline | 3 | 1.5Gi | 1.5 cores |
| Normal | 5 | 2.5Gi | 2.5 cores |
| High | 8 | 4Gi | 4 cores |
| Peak | 10+ | 20Gi+ | 10+ cores |

**Monitoring Growth**:
```bash
# Track request growth over time
kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query_range' \
  --data-urlencode 'query=sum(rate(http_requests_total[5m]))' \
  --data-urlencode 'start=2026-02-13T00:00:00Z' \
  --data-urlencode 'end=2026-02-14T00:00:00Z' \
  --data-urlencode 'step=1h' | jq '.data.result[0].values'
```

---

## Backup & Disaster Recovery

### 4.1 Daily Backup Procedure

```bash
#!/bin/bash
# Daily backup script

BACKUP_DIR="/backups/fingerprint-$(date +%Y%m%d)"
mkdir -p $BACKUP_DIR

echo "Starting daily backup..."

# 1. Backup Kubernetes configuration
echo "Backing up K8s configuration..."
kubectl get all -n fingerprint -o yaml > $BACKUP_DIR/k8s-fingerprint.yaml
kubectl get configmap -n fingerprint -o yaml > $BACKUP_DIR/configmaps.yaml
kubectl get secrets -n fingerprint -o yaml > $BACKUP_DIR/secrets.yaml

# 2. Backup Prometheus data (if needed)
echo "Backing up Prometheus config..."
kubectl get configmap -n prometheus prometheus-config -o yaml > $BACKUP_DIR/prometheus-config.yaml

# 3. Backup application data
echo "Backing up model files..."
kubectl exec -n fingerprint -it <pod-name> -- \
  tar -czf /tmp/models.tar.gz /models/*
kubectl cp fingerprint/<pod-name>:/tmp/models.tar.gz $BACKUP_DIR/models.tar.gz

# 4. Backup Elasticsearch indices
echo "Backing up Elasticsearch snapshots..."
curl -X PUT "localhost:9200/_snapshot/backup/daily-$(date +%Y%m%d)?wait_for_completion=true"

# 5. Verify backup integrity
echo "Verifying backup..."
du -sh $BACKUP_DIR/*

# 6. Archive and compress
tar -czf $BACKUP_DIR.tar.gz $BACKUP_DIR/
rm -rf $BACKUP_DIR

# 7. Upload to external storage
# gsutil cp $BACKUP_DIR.tar.gz gs://your-backup-bucket/
# OR
# aws s3 cp $BACKUP_DIR.tar.gz s3://your-backup-bucket/

echo "✅ Backup complete: $BACKUP_DIR.tar.gz"
```

**Backup Verification Checklist**:
- [ ] K8s configuration files created
- [ ] ConfigMap and Secrets exported
- [ ] Model files archived
- [ ] Elasticsearch snapshots created
- [ ] File sizes reasonable (not suspiciously small)
- [ ] Archive uploaded to external storage
- [ ] Restore test performed (weekly)

### 4.2 Disaster Recovery: Complete Restoration

#### Pre-Restoration Checklist

```bash
# 1. Verify backup integrity
tar -tzf /backups/fingerprint-20260213.tar.gz | head -20

# 2. Confirm Kubernetes cluster is clean
kubectl get all -n fingerprint  # Should be minimal

# 3. Note any manual configurations not in backup
# Check: Ingress TLS certs, external secrets, DNS records
```

#### Restoration Process

```bash
# 1. Extract backup
cd /tmp
tar -xzf /backups/fingerprint-20260213.tar.gz
cd fingerprint-20260213

# 2. Create namespace
kubectl create namespace fingerprint

# 3. Restore ConfigMaps and Secrets (IMPORTANT: Do this first)
kubectl apply -f configmaps.yaml
kubectl apply -f secrets.yaml

# 4. Restore model files
tar -xzf models.tar.gz -C /
# OR on a pod
kubectl cp models.tar.gz fingerprint/<pod-name>:/tmp/
kubectl exec -n fingerprint <pod-name> -- tar -xzf /tmp/models.tar.gz

# 5. Restore Kubernetes resources
kubectl apply -f k8s-fingerprint.yaml

# 6. Verify restoration
kubectl get pods -n fingerprint
kubectl get svc -n fingerprint
kubectl get configmap -n fingerprint

# 7. Run smoke tests
kubectl exec -n fingerprint -it <pod-name> -- curl http://localhost:8000/api/v1/models/status

# 8. Monitor logs for errors
kubectl logs -n fingerprint -l app=fingerprint-api --tail=50
```

#### Restoration Validation

```bash
# 1. Check all pods running
READY=$(kubectl get pods -n fingerprint -o jsonpath='{.items[?(@.status.conditions[].status=="True")].metadata.name}' | wc -w)
echo "Ready pods: $READY/3"

# 2. Test API endpoints
kubectl port-forward -n fingerprint svc/fingerprint-api 8080:80 &
PF_PID=$!
sleep 2

curl -X GET http://localhost:8080/api/v1/models/status
curl -X GET http://localhost:8080/health

kill $PF_PID

# 3. Check Elasticsearch data restored
kubectl exec -n logging -it elasticsearch-0 -- \
  curl -s localhost:9200/fingerprint-api-*/_count | jq '.count'

# 4. Verify model accuracy metrics
kubectl logs -n fingerprint -l app=fingerprint-api | grep "model_accuracy"
```

### 4.3 Incremental Recovery (Partial Failure)

#### Pod Loss

```bash
# If single pod crashes, Kubernetes auto-recovers
# Just verify recovery:
kubectl describe pod <crashed-pod> -n fingerprint

# Check for error events
kubectl get events -n fingerprint --sort-by='.lastTimestamp' | grep <pod-name>
```

#### Data Loss

```bash
# 1. Check Elasticsearch backup
curl -X GET localhost:9200/_snapshot/backup/_all

# 2. Restore specific index from snapshot
curl -X POST "localhost:9200/_snapshot/backup/daily-20260213/_restore?wait_for_completion=true" \
  -H 'Content-Type: application/json' \
  -d '{
    "indices": "fingerprint-api-2026.02.13",
    "ignore_unavailable": true,
    "include_global_state": false
  }'

# 3. Verify index restored
curl -X GET "localhost:9200/fingerprint-api-2026.02.13/_count"
```

---

## Performance Tuning

### 5.1 Model Inference Optimization

```yaml
# Deployment environment variables to tune
env:
  - name: INFERENCE_BATCH_SIZE
    value: "32"  # Batch multiple requests for efficiency
  - name: FEATURE_CACHE_SIZE
    value: "1000"  # Cache feature extractions
  - name: MODEL_WORKERS
    value: "4"  # Parallel model inference
```

### 5.2 Memory Optimization

```bash
# Monitor memory usage
kubectl top pods -n fingerprint --containers

# If OOMKilled, increase limit and request:
kubectl set resources deployment fingerprint-api \
  -n fingerprint \
  --requests=memory=768Mi \
  --limits=memory=3Gi

# Java/Python GC tuning (if applicable):
# Add to deployment env:
# - name: PYTHONHASHSEED
#   value: "0"
```

### 5.3 CPU Optimization

```bash
# Check CPU throttling
kubectl exec -n fingerprint -it <pod-name> -- \
  cat /sys/fs/cgroup/cpu/cpu.stat | grep throttled

# If high throttling, increase CPU request
kubectl set resources deployment fingerprint-api \
  -n fingerprint \
  --requests=cpu=750m
```

### 5.4 Network Optimization

```bash
# Test network latency between pods
kubectl run -it --rm network-test --image=busybox -- \
  sh -c 'ping fingerprint-api.fingerprint.svc.cluster.local'

# Check service DNS latency
kubectl run -it --rm dns-test --image=busybox -- \
  sh -c 'time nslookup fingerprint-api.fingerprint'

# Monitor network throughput
kubectl exec -n fingerprint -it <pod-name> -- \
  iftop -n  # If installed
```

---

## Health Checks & Diagnostics

### 6.1 Liveness Probe Testing

```bash
# Manually test liveness probe
kubectl exec -n fingerprint -it <pod-name> -- \
  curl -v http://localhost:8000/health

# Expected: HTTP 200 OK
```

### 6.2 Readiness Probe Testing

```bash
# Manually test readiness probe
kubectl exec -n fingerprint -it <pod-name> -- \
  curl -v http://localhost:8000/api/v1/models/status

# Expected: HTTP 200 with models loaded
```

### 6.3 Comprehensive Health Diagnostic

```bash
#!/bin/bash
# Complete health diagnostic script

echo "=== Fingerprint API Health Diagnostic ===" 
echo "Time: $(date)"

# 1. Pod Status
echo -e "\n1. Pod Status:"
kubectl get pods -n fingerprint -o wide

# 2. Service Status
echo -e "\n2. Service Status:"
kubectl get svc -n fingerprint -o wide

# 3. Endpoints
echo -e "\n3. Service Endpoints:"
kubectl get endpoints -n fingerprint

# 4. Resource Usage
echo -e "\n4. Resource Usage:"
kubectl top pods -n fingerprint

# 5. Pod Events
echo -e "\n5. Recent Pod Events:"
kubectl get events -n fingerprint --sort-by='.lastTimestamp' | tail -20

# 6. Probe Status
echo -e "\n6. Probe Status:"
kubectl get pods -n fingerprint -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}'

# 7. Volume Mounts
echo -e "\n7. Volume Information:"
kubectl get pvc -n fingerprint

# 8. Network Policies
echo -e "\n8. Network Policies:"
kubectl get networkpolicy -n fingerprint

# 9. RBAC
echo -e "\n9. RBAC Check:"
kubectl auth can-i get pods --as=system:serviceaccount:fingerprint:fingerprint-api -n fingerprint

# 10. API Test
echo -e "\n10. API Connectivity Test:"
kubectl run -it --rm api-test --image=curlimages/curl -- \
  sh -c 'curl -s http://fingerprint-api.fingerprint/api/v1/models/status | head -c 100'

echo -e "\n=== Diagnostic Complete ==="
```

---

## Monitoring & Alerting

### 7.1 Key Metrics to Monitor

**Application Metrics**:
- Request rate (requests/second)
- Error rate (% of 5xx responses)
- Latency (P50, P95, P99 in milliseconds)
- Model accuracy (% correct classifications)
- Feature extraction latency

**Infrastructure Metrics**:
- CPU utilization (%)
- Memory utilization (%)
- Network I/O (B/s)
- Disk usage (%)
- Pod restart count

### 7.2 Alert Thresholds

| Alert | Threshold | Duration | Action |
|-------|-----------|----------|--------|
| HighErrorRate | >5% | 5 min | Page on-call |
| HighLatency | P99 > 1s | 5 min | Check logs, scale if needed |
| PodCrashLoop | >3 restarts | 10 min | Investigate logs, check resources |
| HighCPUUsage | >80% | 5 min | Scale or optimize queries |
| HighMemoryUsage | >85% | 5 min | Increase memory limit or investigate leak |
| DiskFull | >85% | 5 min | Page on-call immediately |

### 7.3 Custom Metric Setup

```yaml
# Example: Add custom metric query
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-custom-rules
  namespace: prometheus
data:
  custom_rules.yml: |
    groups:
      - name: custom_metrics
        interval: 30s
        rules:
          - record: fingerprint:inference_p99
            expr: histogram_quantile(0.99, fingerprint_api_inference_duration_seconds_bucket)
          
          - record: fingerprint:business_metric
            expr: rate(fingerprint_api_successful_identifications[5m])
```

---

## Common Issues & Solutions

### 8.1 Memory Leak Investigation

**Symptom**: Memory usage gradually increasing despite stable request rate

```bash
# 1. Check current memory usage
kubectl top pods -n fingerprint

# 2. Check GC logs
kubectl logs -n fingerprint <pod-name> | grep -i "garbage\|gc"

# 3. Get memory profile
kubectl exec -n fingerprint <pod-name> -- \
  curl http://localhost:8000/debug/pprof/heap > heap.dump

# 4. Analyze with profiling tool
# For Python:
# python3 -m pstats heap.dump

# 5. If confirmed leak, restart pod
kubectl delete pod <pod-name> -n fingerprint

# 6. Monitor for recurrence
watch 'kubectl top pods -n fingerprint'
```

### 8.2 Model Loading Failure

**Symptom**: Readiness probe failing, "models not loaded" error

```bash
# 1. Check model files exist
kubectl exec -n fingerprint <pod-name> -- ls -la /models/

# 2. Check model file permissions
kubectl exec -n fingerprint <pod-name> -- file /models/*.pkl

# 3. Verify ConfigMap mount
kubectl get configmap fingerprint-config -n fingerprint -o yaml

# 4. Check pod logs
kubectl logs -n fingerprint <pod-name> | grep -i "model\|load"

# 5. Test model load manually
kubectl exec -n fingerprint <pod-name> -- python3 -c \
  "import pickle; pickle.load(open('/models/family_classifier.pkl', 'rb'))"

# 6. If files missing, restore from backup or rebuild
```

### 8.3 High Latency Spikes

**Symptom**: Occasional latency spikes to >5 seconds

```bash
# 1. Check if concurrent with high throughput
kubectl exec -n prometheus -it prometheus-pod -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=rate(http_requests_total[1m])'

# 2. Check GC pauses (for Python/Java)
kubectl logs -n fingerprint <pod-name> | grep -i "pause\|gc"

# 3. Check network packet loss
kubectl exec -n fingerprint <pod-name> -- netstat -s | grep -i "drop\|error"

# 4. Check CPU throttling
kubectl exec -n fingerprint <pod-name> -- \
  cat /sys/fs/cgroup/cpu/cpu.stat | grep throttled

# 5. If GC pauses: Increase heap size or adjust GC parameters
# If CPU throttling: Increase CPU requests
# If network issues: Check cluster networking
```

### 8.4 Elasticsearch Disk Full

**Symptom**: Logs not ingesting, "disk full" error

```bash
# 1. Check disk usage
kubectl exec -n logging -it elasticsearch-0 -- df -h /usr/share/elasticsearch/data

# 2. List indices and sizes
kubectl exec -n logging -it elasticsearch-0 -- \
  curl -s localhost:9200/_cat/indices?h=index,store.size,docs.count

# 3. Delete old indices (if safe)
kubectl exec -n logging -it elasticsearch-0 -- \
  curl -X DELETE localhost:9200/fingerprint-api-2026.01.*?s

# 4. Shrink large indices
kubectl exec -n logging -it elasticsearch-0 -- \
  curl -X POST localhost:9200/fingerprint-api-2026.02/_shrink/fingerprint-api-2026.02-shrink

# 5. Enable log rotation in Logstash
kubectl set env deployment logstash -n logging LOG_ROTATION_SIZE=100M

# 6. Increase PVC size (if using persistent volumes)
```

---

## Operations Checklist

### 9.1 Pre-Deployment Checklist

- [ ] All tests passing locally
- [ ] Code reviewed and approved
- [ ] Staging deployment validated
- [ ] Performance benchmarks acceptable
- [ ] Security scan completed
- [ ] Backup taken before deployment
- [ ] On-call engineer informed
- [ ] Rollback procedure documented
- [ ] Communication plan ready

### 9.2 Post-Deployment Checklist

- [ ] All pods running and healthy
- [ ] Service endpoints active
- [ ] Metrics flowing to Prometheus
- [ ] Logs flowing to Elasticsearch
- [ ] Smoke tests passing
- [ ] No error spikes in first 5 minutes
- [ ] Alert thresholds adjusted if needed
- [ ] Team notified of deployment complete
- [ ] Documentation updated

### 9.3 Weekly Operations Review

Every Monday morning:
- [ ] Review incident logs from past week
- [ ] Check resource usage trends
- [ ] Update capacity planning forecasts
- [ ] Review alert threshold effectiveness
- [ ] Test disaster recovery procedures
- [ ] Update runbooks with new learnings
- [ ] Plan for upcoming maintenance windows

### 9.4 Monthly Maintenance Window

First Tuesday of each month (if needed):
- [ ] Apply security patches
- [ ] Update Kubernetes cluster (if required)
- [ ] Optimize database indices
- [ ] Archive old logs
- [ ] Review security scan results
- [ ] Capacity planning session
- [ ] Performance optimization review

---

## Emergency Contacts & Escalation

### 10.1 Escalation Matrix

| Time | Severity | Contacts |
|------|----------|----------|
| Business hours (9-5) | P1 | On-call → SRE Lead → Manager |
| Business hours (9-5) | P2 | On-call → DevOps Lead |
| After hours | P1 | On-call → SRE Lead (escalate immediately) |
| After hours | P2 | On-call (resolve within 1 hour or escalate) |

### 10.2 Contact Information Template

```
On-Call Engineer: [Name] [Phone] [Slack]
Backup On-Call: [Name] [Phone] [Slack]
SRE Lead: [Name] [Phone] [Slack]
Manager: [Name] [Phone] [Slack]
Infrastructure Team: [Channel/Email]
Security Team: [Channel/Email]
```

### 10.3 Alert Routing

**Slack Channels**:
- #fingerprint-api-alerts - All alerts
- #fingerprint-api-critical - P1 only
- #fingerprint-api-incidents - Any open incidents

**PagerDuty**:
- Fingerprint API P1 escalation
- On-call rotation: Weekly

**Email**:
- ops-team@example.com - Daily digest
- sre-lead@example.com - Critical alerts

---

## Appendices

### A. Useful Commands

```bash
# Get pod information
kubectl get pods -n fingerprint -o wide

# Port-forward for testing
kubectl port-forward -n fingerprint svc/fingerprint-api 8080:80

# Access logs
kubectl logs -n fingerprint <pod-name> -f

# Execute command in pod
kubectl exec -n fingerprint <pod-name> -- <command>

# Port-forward to Prometheus
kubectl port-forward -n prometheus svc/prometheus 9090:9090

# Port-forward to Grafana
kubectl port-forward -n grafana svc/grafana 3000:3000

# Port-forward to Kibana
kubectl port-forward -n logging svc/kibana 5601:5601
```

### B. Important Paths in Container

```
/models/                    - Trained ML models
/tmp/                       - Temporary files, request queue
/etc/ssl/certs/             - SSL certificates
/app/config/                - Application configuration
```

### C. Key Configuration Files

```
Kubernetes:
  k8s/base/deployment.yaml
  k8s/base/configmap.yaml

Monitoring:
  monitoring/prometheus/prometheus.yml
  monitoring/prometheus/alert_rules.yml

Logging:
  monitoring/elk/logstash/fingerprint-api.conf
  monitoring/elk/elasticsearch/elasticsearch.yml
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-13  
**Next Review**: 2026-03-13  
**Owner**: DevOps / SRE Team  

For questions or updates, please contact the SRE lead or submit a PR to update this documentation.
