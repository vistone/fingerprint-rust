# Troubleshooting Guide & FAQ

**Document**: Fingerprint API Troubleshooting Reference  
**Version**: 1.0  
**Last Updated**: 2026-02-13  

---

## Quick Reference

### Most Common Issues (and Quick Fixes)

| Issue | Symptom | Fix |
|-------|---------|-----|
| Pods not running | `CrashBackOff` status | `kubectl logs <pod>` to see error, fix and restart |
| High latency | P99 > 1 second | Scale: `kubectl scale deployment fingerprint-api --replicas=5` |
| Models not loaded | Readiness probe failing | Check ConfigMap: `kubectl get cm fingerprint-config -o yaml` |
| Disk full | Logs not ingesting | Delete old indices: `curl -X DELETE localhost:9200/fingerprint-api-2026.01.*` |
| High error rate | >5% 5xx responses | Check logs: `kubectl logs -l app=fingerprint-api --tail=200` |

---

## Section 1: Pod & Deployment Issues

### 1.1 Pod Status: CrashBackOff

**What it means**: Pod is repeatedly crashing and restarting

**How to diagnose**:
```bash
# Get pod status
kubectl describe pod <pod-name> -n fingerprint

# Check logs from before crash
kubectl logs <pod-name> -n fingerprint --previous

# Check all container logs
kubectl logs <pod-name> -n fingerprint --all-containers=true --previous
```

**Common causes and fixes**:

#### Cause 1: Memory exhausted (OOMKilled)
```bash
# Check if OOMKilled
kubectl get pod <pod-name> -n fingerprint -o jsonpath='{.status.containerStatuses[0].lastState.terminated.reason}'
# Output: OOMKilled

# Fix: Increase memory limit
kubectl set resources deployment fingerprint-api -n fingerprint --limits=memory=2Gi --requests=memory=768Mi

# Verify change
kubectl get pod <pod-name> -n fingerprint -o yaml | grep -A 5 resources:
```

#### Cause 2: Application error on startup
```bash
# Check startup logs
kubectl logs <pod-name> -n fingerprint --previous | head -50

# Common errors:
# - "ModuleNotFoundError": Missing Python dependency
# - "FileNotFoundError": Model file not found
# - "PermissionDenied": File permissions issue on mount

# Fix based on error type
# If model files: verify ConfigMap mount
kubectl get cm fingerprint-config -n fingerprint -o yaml | grep -A 5 data

# If dependency: check requirements.txt in image
```

#### Cause 3: Readiness probe failing
```bash
# Test the readiness probe endpoint
kubectl exec -n fingerprint <pod-name> -- curl http://localhost:8000/api/v1/models/status

# Expected output: HTTP 200 with JSON response
# If fails: models not loaded, see Section 2.2 below
```

### 1.2 Pod Status: Pending

**What it means**: Pod is waiting to be scheduled on a node

**How to diagnose**:
```bash
# Check pod events
kubectl describe pod <pod-name> -n fingerprint | grep -A 10 Events

# Common messages:
# - "Insufficient memory": No node with enough free memory
# - "Insufficient cpu": No node with enough CPU
# - "No nodes available": All nodes full or unavailable
```

**How to fix**:

#### Option 1: Lower resource requests
```bash
kubectl set resources deployment fingerprint-api -n fingerprint \
  --requests=cpu=250m,memory=256Mi
```

#### Option 2: Add more nodes to cluster
```bash
# Contact infrastructure team to scale cluster
# Then check if pod gets scheduled
kubectl get pods -n fingerprint -w
```

#### Option 3: Check node availability
```bash
# List nodes and resource usage
kubectl top nodes

# Check for tainted nodes (that reject pods)
kubectl describe nodes | grep -i taint
```

### 1.3 Deployment Stuck in Rollout

**What it means**: New version deployment is not completing

**How to diagnose**:
```bash
# Check rollout status
kubectl rollout status deployment/fingerprint-api -n fingerprint --timeout=30s

# If stuck, check pod status
kubectl get pods -n fingerprint -o wide

# Check recent events
kubectl get events -n fingerprint --sort-by='.lastTimestamp' | tail -20
```

**How to fix**:

```bash
# Option 1: Increase timeout for probes
kubectl patch deployment fingerprint-api -n fingerprint --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/containers/0/readinessProbe/initialDelaySeconds", "value": 60}]'

# Option 2: Rollback to previous version
kubectl rollout undo deployment/fingerprint-api -n fingerprint

# Option 3: Manual scale fix
kubectl scale deployment fingerprint-api -n fingerprint --replicas=0
sleep 10
kubectl scale deployment fingerprint-api -n fingerprint --replicas=3
```

---

## Section 2: Model & Feature Issues

### 2.1 Models Not Loaded

**Symptom**: Readiness probe failing, `api_models_loaded == 0`

```bash
# Test the status endpoint
kubectl exec -n fingerprint <pod-name> -- curl http://localhost:8000/api/v1/models/status

# Expected: HTTP 200 with model list
# If fails or empty: models not loading
```

**How to diagnose**:
```bash
# 1. Check if model files exist
kubectl exec -n fingerprint <pod-name> -- ls -la /models/

# 2. Check file sizes (sanity check)
kubectl exec -n fingerprint <pod-name> -- du -sh /models/*

# 3. Check pod logs for model loading errors
kubectl logs -n fingerprint <pod-name> | grep -i "model\|load\|error"

# 4. Check if ConfigMap is mounted
kubectl get configmap fingerprint-config -n fingerprint -o yaml | head -20

# 5. Verify ConfigMap volume mount in deployment
kubectl get deployment fingerprint-api -n fingerprint -o yaml | grep -A 10 volumeMounts
```

**How to fix**:

#### Issue: Models missing from ConfigMap
```bash
# If models are not in ConfigMap, add them
# 1. Verify they exist locally
ls -la /path/to/models/

# 2. Update ConfigMap (if small enough, <1MB each)
kubectl create configmap fingerprint-models \
  --from-file=/path/to/models/ \
  -n fingerprint \
  --dry-run=client -o yaml | kubectl apply -f -

# 3. Mount ConfigMap in deployment
kubectl set env deployment fingerprint-api -n fingerprint \
  MODELS_PATH=/models
kubectl patch deployment fingerprint-api -n fingerprint \
  --type='json' \
  -p='[{"op": "add", "path": "/spec/template/spec/volumes/-", "value": {"name": "models", "configMap": {"name": "fingerprint-models"}}}]'

# 4. Restart deployment
kubectl rollout restart deployment/fingerprint-api -n fingerprint
```

#### Issue: ConfigMap corrupted or incomplete
```bash
# 1. Backup current ConfigMap
kubectl get configmap fingerprint-config -n fingerprint -o yaml > configmap-backup.yaml

# 2. Delete and recreate
kubectl delete configmap fingerprint-config -n fingerprint
kubectl create configmap fingerprint-config \
  --from-literal=log_level=info \
  --from-literal=workers=4 \
  -n fingerprint

# 3. Restart deployment to pick up new config
kubectl rollout restart deployment/fingerprint-api -n fingerprint
```

### 2.2 Feature Extraction Errors

**Symptom**: Inference requests failing with "Invalid feature dimensions"

**How to diagnose**:
```bash
# Check feature extraction logs
kubectl logs -n fingerprint <pod-name> | grep -i "feature\|dimension"

# Test feature extraction manually
kubectl exec -n fingerprint <pod-name> -- python3 -c \
  "from features import TLSFeatureExtractor; print(TLSFeatureExtractor().extract({...}))"

# Check Prometheus metrics for feature extraction latency
kubectl exec -n prometheus prometheus-0 -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=fingerprint_api_feature_extraction_duration_seconds'
```

**Common causes**:

#### Issue: Input data missing required fields
```bash
# Check what the inference request looks like
curl -X POST http://fingerprint-api/api/v1/fingerprint/identify \
  -H 'Content-Type: application/json' \
  -d '{...}' -v

# Required fields:
# - tls_version
# - supported_ciphers
# - extensions
# - curves
# - signature_algorithms

# Fix: Ensure client sends all required fields
```

#### Issue: Model expects different feature count
```bash
# Check expected feature count
kubectl exec -n fingerprint <pod-name> -- python3 -c \
  "import pickle; m = pickle.load(open('/models/family_classifier.pkl', 'rb')); print(m.n_features_in_)"

# Compare with actual extraction
kubectl exec -n fingerprint <pod-name> -- python3 -c \
  "from features import TLSFeatureExtractor; f = TLSFeatureExtractor().extract({...}); print(len(f))"

# If mismatch: retrain model with updated feature extraction
```

---

## Section 3: Performance Issues

### 3.1 High Latency

**Symptom**: Response times >1 second for identify endpoint

**Step-by-step diagnosis**:

```bash
# 1. Check if latency is consistent
kubectl exec -n prometheus prometheus-0 -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{endpoint="/api/v1/fingerprint/identify"}[5m]))'

# 2. Break down latency by component
kubectl exec -n prometheus prometheus-0 -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=histogram_quantile(0.99, rate(fingerprint_api_feature_extraction_duration_seconds_bucket[5m]))'

kubectl exec -n prometheus prometheus-0 -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=histogram_quantile(0.99, rate(fingerprint_api_inference_duration_seconds_bucket[5m]))'

# 3. Check resource utilization
kubectl top pods -n fingerprint
kubectl top nodes
```

**Likely causes and fixes**:

| Cause | Diagnostic | Fix |
|-------|-----------|-----|
| Model inference slow | `fingerprint_api_inference_duration_seconds > 500ms` | Profile model, consider model optimization |
| Resource contention | CPU >80% or Memory >85% | Scale: `kubectl scale deployment ... --replicas=6` |
| Queue buildup | High request queue depth | Increase workers or scale out |
| Network latency | Check inter-pod latency | Run network diagnostic: `kubectl run -it debug --image=busybox -- ping fingerprint-api` |
| GC pauses | JVM/Python GC logs show long pauses | Tune GC parameters or increase heap/memory |

**Quick fix - Scale horizontally**:
```bash
# Check current replica count
kubectl get deployment fingerprint-api -n fingerprint -o jsonpath='{.spec.replicas}'

# Scale up
kubectl scale deployment fingerprint-api -n fingerprint --replicas=6

# Monitor latency improvement
watch 'kubectl exec -n prometheus prometheus-0 -- curl -s "http://localhost:9090/api/v1/query?query=histogram_quantile(0.99,rate(http_request_duration_seconds_bucket[5m]))"'
```

### 3.2 High CPU Usage

**Symptom**: CPU consistently >70%

```bash
# Check which pods using most CPU
kubectl top pods -n fingerprint

# Check overall node CPU
kubectl top nodes
```

**Diagnosis & fix**:

```bash
# 1. Determine if legitimate high load
kubectl exec -n prometheus prometheus-0 -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=rate(http_requests_total[5m])'

# Compare:
# - High requests + high CPU = normal, scale out
# - Low requests + high CPU = something inefficient

# 2. If low requests causing high CPU:
# Profile with pprof (Python)
kubectl exec -n fingerprint <pod-name> -- python3 -m cProfile app.py

# 3. If high requests, scale out
kubectl scale deployment fingerprint-api -n fingerprint --replicas=6
```

### 3.3 Memory Leak (Gradually Increasing Memory)

**Symptom**: Memory usage growing over hours despite stable request rate

```bash
# Monitor memory trend
watch -n 60 'kubectl top pods -n fingerprint | grep fingerprint-api'

# Check for memory spike at specific time
kubectl exec -n prometheus prometheus-0 -- \
  curl -s 'http://localhost:9090/api/v1/query_range' \
  --data-urlencode 'query=container_memory_usage_bytes{pod=~"fingerprint-api.*"}' \
  --data-urlencode 'start=2026-02-13T00:00:00Z' \
  --data-urlencode 'end=2026-02-14T00:00:00Z' \
  --data-urlencode 'step=1m'
```

**How to fix**:

```bash
# Option 1: Restart pods to clear memory
kubectl delete pods -n fingerprint -l app=fingerprint-api

# Option 2: Investigate memory leak (if leak confirmed)
# Get memory dump
kubectl exec -n fingerprint <pod-name> -- python3 -m tracemalloc app.py

# Option 3: Increase memory limit temporarily
kubectl set resources deployment fingerprint-api -n fingerprint --limits=memory=3Gi

# Option 4: Set memory requests to trigger pod restarts due to memory pressure
# This allows automatic pod restart when memory exceeds limit
kubectl patch deployment fingerprint-api -n fingerprint --type='json' \
  -p='[{"op": "add", "path": "/spec/template/spec/containers/0/lifecycle/preStop", "value": {"exec": {"command": ["/bin/sh", "-c", "sleep 15"]}}}]'
```

---

## Section 4: Network & Connectivity

### 4.1 Service Unreachable from Outside Cluster

**Symptom**: Cannot connect to API from external client

```bash
# 1. Check if service exists and has endpoints
kubectl get svc -n fingerprint fingerprint-api
kubectl get endpoints -n fingerprint fingerprint-api

# 2. Check ingress configuration
kubectl get ingress -n fingerprint
kubectl describe ingress fingerprint-api-ingress -n fingerprint

# 3. Check ingress controller status
kubectl get pods -n ingress-nginx
```

**How to fix**:

#### Issue: Service has no endpoints
```bash
# Check if pods are running
kubectl get pods -n fingerprint -o wide

# If pods pending/crashing, fix pod issues (see Section 1)

# If pods running but no endpoints, check service selector
kubectl get svc fingerprint-api -n fingerprint -o yaml | grep -A 5 selector

# Verify pods have matching labels
kubectl get pods -n fingerprint -L app
```

#### Issue: Ingress not configured properly
```bash
# Check ingress rules
kubectl get ingress fingerprint-api-ingress -n fingerprint -o yaml

# Verify backend service exists
kubectl get svc fingerprint-api -n fingerprint

# Check ingress class
kubectl get ingressclass

# Verify ingress controller is running
kubectl get pods -n ingress-nginx -o wide
```

#### Issue: DNS not resolving
```bash
# Test from a pod
kubectl run -it --rm dns-test --image=busybox -- nslookup fingerprint-api.fingerprint.svc.cluster.local

# If fails, check Kubernetes DNS
kubectl get pods -n kube-system | grep coredns
kubectl logs -n kube-system -l k8s-app=kube-dns
```

### 4.2 Request Timeouts

**Symptom**: Client requests timing out, but server responding

```bash
# Check server-side latency (should be <1s)
curl -w "@response.txt" https://fingerprint-api/api/v1/models/status 2>&1 | tail -5

# Check network latency
kubectl run -it --rm ping-test --image=busybox -- ping fingerprint-api.fingerprint.svc.cluster.local
kubectl run -it --rm traceroute-test --image=busybox -- traceroute fingerprintapi.fingerprint.svc.cluster.local
```

**How to fix**:

```bash
# 1. If server latency high, see Section 3.1
# 2. If network latency high:

# Check ingress timeout settings
kubectl get ingress fingerprint-api-ingress -n fingerprint -o yaml | grep -i timeout

# Increase timeout if needed:
kubectl patch ingress fingerprint-api-ingress -n fingerprint --type='json' \
  -p='[{"op": "replace", "path": "/metadata/annotations/nginx~1proxy-connect-timeout", "value": "60s"}]'

# 3. Check pod termination grace period
kubectl get deployment fingerprint-api -n fingerprint -o yaml | grep terminationGracePeriodSeconds

# Increase if needed
kubectl patch deployment fingerprint-api -n fingerprint --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/terminationGracePeriodSeconds", "value": 60}]'
```

---

## Section 5: Logging & Monitoring Issues

### 5.1 Logs Not Appearing in Kibana

**Symptom**: Pod's logs available via `kubectl logs` but not in Kibana

```bash
# 1. Check if Logstash is running
kubectl get pods -n logging -l app=logstash

# 2. Check Logstash logs
kubectl logs -n logging -l app=logstash --tail=100 | grep -i "error\|fingerprint"

# 3. Check Elasticsearch is receiving data
curl -s -u elastic:password localhost:9200/fingerprint-api-*/_count | jq .

# 4. Check if indices are created
curl -s -u elastic:password localhost:9200/_cat/indices | grep fingerprint
```

**How to fix**:

#### Issue: Logstash not running
```bash
# Restart Logstash
kubectl rollout restart statefulset logstash -n logging

# Check if it starts successfully
kubectl logs -n logging -l app=logstash --tail=50
```

#### Issue: Elasticsearch indices not created
```bash
# Check FLastic connectivity
kubectl logs -n logging -l app=logstash | grep -i "connection\|refused"

# If error, check Elasticsearch service
kubectl get svc -n logging elasticsearch

# Manually create index
curl -X PUT "localhost:9200/fingerprint-api-2026.02.13" -u elastic:password
```

#### Issue: Logstash parser not matching logs
```bash
# Check Logstash configuration
kubectl get configmap -n logging logstash-config -o yaml

# Test grok pattern
# Use free online tool: https://grokdebug.herokuapp.com/

# If pattern wrong, update ConfigMap and restart
kubectl set env deployment logstash -n logging \
  LOGSTASH_CONFIG="<new config>"
kubectl rollout restart deployment logstash -n logging
```

### 5.2 Prometheus Not Scraping Metrics

**Symptom**: Prometheus shows 0 targets or "DOWN" status

```bash
# Check Prometheus targets
kubectl exec -n prometheus prometheus-0 -- \
  curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets | length'

# Check specific target status
kubectl exec -n prometheus prometheus-0 -- \
  curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.labels.job=="fingerprint-api")'
```

**How to fix**:

```bash
# 1. Check Prometheus config
kubectl get configmap -n prometheus prometheus-config -o yaml | grep -A 20 "fingerprint-api"

# 2. Verify service discovery
kubectl get pods -n fingerprint -l app=fingerprint-api --show-labels

# 3. Check if metrics endpoint is accessible
kubectl exec -n fingerprint <pod-name> -- curl http://localhost:8000/metrics | head -20

# 4. Check Kubernetes RBAC for Prometheus
kubectl auth list-k8s-api-server-local --as=system:serviceaccount:prometheus:prometheus

# 5. Reload Prometheus config
kubectl exec -n prometheus prometheus-0 -- kill -HUP 1

# Or restart Prometheus
kubectl rollout restart deployment prometheus -n prometheus
```

---

## Section 6: Database & Storage Issues

### 6.1 Model Files Corrupted

**Symptom**: Models loaded but inference producing garbage results

```bash
# Verify model file integrity
kubectl exec -n fingerprint <pod-name> -- python3 -c \
  "import pickle; m = pickle.load(open('/models/family_classifier.pkl', 'rb')); print(type(m))"

# Check model predictions on known sample
kubectl exec -n fingerprint <pod-name> -- python3 -c \
  "import pickle; m = pickle.load(open('/models/family_classifier.pkl', 'rb')); \
   print(m.predict([[1,2,3,4,5]]))"

# Expected: Should return valid class (0-10 for family classifier)
```

**How to fix**:

```bash
# 1. Restore models from backup
kubectl cp /path/to/backup/models <pod-name>:/models -n fingerprint
```

# 2. Trigger model reload
kubectl rollout restart deployment fingerprint-api -n fingerprint

# 3. Verify metrics show models loaded
kubectl exec -n prometheus prometheus-0 -- \
  curl -s 'http://localhost:9090/api/v1/query' \
  --data-urlencode 'query=fingerprint_api_models_loaded'
```

### 6.2 PVC Full or Unavailable

**Symptom**: Pod stuck in `Pending` due to PVC issues

```bash
# Check PVC status
kubectl get pvc -n fingerprint

# Check for errors
kubectl describe pvc <pvc-name> -n fingerprint

# Check available disk space
kubectl exec -n fingerprint <pod-name> -- df -h /data
```

**How to fix**:

```bash
# Option 1: Expand PVC (if storage supports it)
kubectl patch pvc models -n fingerprint -p '{"spec":{"resources":{"requests":{"storage":"10Gi"}}}}'

# Option 2: Clean up unnecessary files
kubectl exec -n fingerprint <pod-name> -- rm -rf /data/temp/*

# Option 3: Delete old snapshots/backups on PVC
kubectl exec -n fingerprint <pod-name> -- ls -la /data/
kubectl exec -n fingerprint <pod-name> -- rm /data/old-backup-*

# Verify space recovered
kubectl exec -n fingerprint <pod-name> -- df -h
```

---

## Section 7: Configuration Issues

### 7.1 Configuration Changes Not Applied

**Symptom**: Changed ConfigMap but pods still using old configuration

```bash
# Verify ConfigMap was updated
kubectl get configmap fingerprint-config -n fingerprint -o yaml | grep -A 3 "log_level:"

# Check if pod has new mounting
kubectl exec <pod-name> -n fingerprint -- cat /etc/config/app.conf
```

**How to fix**:

```bash
# ConfigMaps are not automatically reloaded, must restart pods
kubectl rollout restart deployment fingerprint-api -n fingerprint

# Verify new config loaded
kubectl exec <pod-name> -n fingerprint -- cat /etc/config/app.conf

# If still old, check ConfigMap volume mount
kubectl describe pod <pod-name> -n fingerprint | grep -A 5 Mounts
```

### 7.2 Secrets Not Mounted

**Symptom**: Pod unable to authenticate to external services

```bash
# Check if secret exists
kubectl get secrets -n fingerprint

# Check if mounted in pod
kubectl exec -n fingerprint <pod-name> -- ls -la /etc/secrets/

# Check pod volume mounts
kubectl describe pod <pod-name> -n fingerprint | grep -A 10 Mounts
```

**How to fix**:

```bash
# 1. Create secret if missing
kubectl create secret generic app-secrets \
  --from-literal=api_key=<value> \
  -n fingerprint

# 2. Mount secret in deployment
kubectl patch deployment fingerprint-api -n fingerprint --type='json' \
  -p='[{"op": "add", "path": "/spec/template/spec/volumes/-", "value": {"name": "secrets", "secret": {"secretName": "app-secrets"}}}]'

# 3. Mount in container
kubectl patch deployment fingerprint-api -n fingerprint --type='json' \
  -p='[{"op": "add", "path": "/spec/template/spec/containers/0/volumeMounts/-", "value": {"name": "secrets", "mountPath": "/etc/secrets", "readOnly": true}}]'

# 4. Restart deployment
kubectl rollout restart deployment fingerprint-api -n fingerprint
```

---

## FAQ

### Q: How do I check if an alert is firing?

```bash
# Check Alertmanager
kubectl exec -n prometheus alertmanager-0 -- curl -s http://localhost:9093/api/v1/alerts | jq '.data'

# Or check Prometheus
kubectl exec -n prometheus prometheus-0 -- curl -s http://localhost:9090/api/v1/alerts | jq '.data.alerts'
```

### Q: How do I manually run tests for the API?

```bash
# Port-forward
kubectl port-forward -n fingerprint svc/fingerprint-api 8080:80

# Test health endpoint
curl http://localhost:8080/health

# Test models status
curl http://localhost:8080/api/v1/models/status

# Test inference (get sample data first)
curl -X POST http://localhost:8080/api/v1/fingerprint/identify \
  -H 'Content-Type: application/json' \
  -d '{...}'
```

### Q: How do I scale the deployment manually?

```bash
# Scale to specific replicas
kubectl scale deployment fingerprint-api -n fingerprint --replicas=5

# Monitor pods coming up
kubectl get pods -n fingerprint -w
```

### Q: How do I check resource limits?

```bash
# View current limits
kubectl get deployment fingerprint-api -n fingerprint -o yaml | grep -A 10 "resources:"

# Update limits
kubectl set resources deployment fingerprint-api -n fingerprint \
  --requests=cpu=750m,memory=768Mi \
  --limits=cpu=2000m,memory=2Gi
```

### Q: How do I restart a deployment without downtime?

```bash
# Rolling restart (respects maxUnavailable)
kubectl rollout restart deployment fingerprint-api -n fingerprint

# Monitor rollout
kubectl rollout status deployment/fingerprint-api -n fingerprint
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-13  
**Owner**: SRE / DevOps Team  

For additional help, contact: sre-support@example.com
