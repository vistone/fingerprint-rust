# Multi-Region Deployment Guide

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



**Phase**: 9.1  
**Status**: Implementation in progress  
**Estimated Completion**: 6 hours  

---

## 📋 Overview

This guide explains how to deploy the Fingerprint API across multiple regions for high availability and low latency globally.

---

## 🏗️ Architecture

### Regions

| Region | Cluster | Purpose | Replicas | Latency Target |
|--------|---------|---------|----------|-----------------|
| **US-EAST-1** | Primary | Main serving region | 5-20 | <100ms |
| **EU-WEST-1** | Secondary | Fallback + Europe traffic | 3-15 | <150ms |
| **AP-SOUTHEAST-1** | Tertiary | Fallback + APAC traffic | 2-10 | <200ms |

### Traffic Distribution

```
User Request
    ↓
Global Load Balancer (GeoDNS / Route53)
    ↓
┌───────────────────┬──────────────────┬──────────────────┐
50% US-EAST-1       30% EU-WEST-1      20% AP-SOUTHEAST-1
└───────────────────┴──────────────────┴──────────────────┘
```

---

## 🔧 Prerequisites

1. **Three Kubernetes clusters** (one per region)
   ```bash
   # Example: GKE clusters
   gcloud container clusters create fingerprint-api-us-east \
     --region us-east1 --num-nodes 3
   gcloud container clusters create fingerprint-api-eu-west \
     --region europe-west1 --num-nodes 3
   gcloud container clusters create fingerprint-api-ap-southeast \
     --region asia-southeast1 --num-nodes 2
   ```

2. **Configure kubeconfig for each cluster**
   ```bash
   gcloud container clusters get-credentials fingerprint-api-us-east --region us-east1
   gcloud container clusters get-credentials fingerprint-api-eu-west --region europe-west1
   gcloud container clusters get-credentials fingerprint-api-ap-southeast --region asia-southeast1
   
   # Rename contexts
   kubectl config rename-context gke_PROJECT_us-east1_fingerprint-api-us-east us-east-1
   kubectl config rename-context gke_PROJECT_europe-west1_fingerprint-api-eu-west eu-west-1
   kubectl config rename-context gke_PROJECT_asia-southeast1_fingerprint-api-ap-southeast ap-southeast-1
   ```

3. **Install Istio across all clusters**
   ```bash
   # Download Istio
   curl -L https://istio.io/downloadIstio | sh -
   cd istio-1.18.0  # Latest version
   
   # Install on each cluster
   for region in us-east-1 eu-west-1 ap-southeast-1; do
     kubectl --context=$region create namespace istio-system
     istioctl install --context=$region -y
   done
   ```

4. **Storage buckets for model replication**
   ```bash
   # Create regional buckets
   gsutil mb -l us-east1 gs://fingerprint-models-us-east-1
   gsutil mb -l europe-west1 gs://fingerprint-models-eu-west-1
   gsutil mb -l asia-southeast1 gs://fingerprint-models-ap-southeast-1
   ```

5. **Redis cluster for distributed caching**
   ```bash
   # Will be deployed in Phase 9.3
   # Placeholder for now
   ```

---

## 📦 Deployment Steps

### Step 1: Deploy US-EAST-1 (Primary Region)

```bash
# Set context
kubectl config use-context us-east-1

# Create namespace
kubectl create namespace fingerprint-api
kubectl label namespace fingerprint-api istio-injection=enabled

# Deploy using kustomize
kubectl apply -k k8s/overlays/us-east-1/

# Verify deployment
kubectl rollout status deployment/fingerprint-api -n fingerprint-api
kubectl get pods -n fingerprint-api
```

**Expected Output**:
```
NAME                              READY   STATUS    RESTARTS   AGE
us-east-1-fingerprint-api-xyz     1/1     Running   0          2m
us-east-1-fingerprint-api-abc     1/1     Running   0          2m
us-east-1-fingerprint-api-def     1/1     Running   0          2m
```

### Step 2: Deploy EU-WEST-1 (Secondary Region)

```bash
# Switch context
kubectl config use-context eu-west-1

# Create namespace
kubectl create namespace fingerprint-api
kubectl label namespace fingerprint-api istio-injection=enabled

# Deploy using kustomize
kubectl apply -k k8s/overlays/eu-west-1/

# Verify deployment
kubectl rollout status deployment/fingerprint-api -n fingerprint-api
kubectl get pods -n fingerprint-api
```

### Step 3: Deploy AP-SOUTHEAST-1 (Tertiary Region)

```bash
# Switch context
kubectl config use-context ap-southeast-1

# Create namespace
kubectl create namespace fingerprint-api
kubectl label namespace fingerprint-api istio-injection=enabled

# Deploy using kustomize
kubectl apply -k k8s/overlays/ap-southeast-1/

# Verify deployment
kubectl rollout status deployment/fingerprint-api -n fingerprint-api
kubectl get pods -n fingerprint-api
```

---

### Step 4: Configure Service Mesh

Deploy Istio VirtualService and DestinationRule to all regions:

```bash
# Deploy to US-EAST
kubectl config use-context us-east-1
kubectl apply -f k8s/networking/istio/virtualservice.yaml
kubectl apply -f k8s/networking/istio/gateway.yaml

# Deploy to EU-WEST
kubectl config use-context eu-west-1
kubectl apply -f k8s/networking/istio/virtualservice.yaml
kubectl apply -f k8s/networking/istio/gateway.yaml

# Deploy to AP-SOUTHEAST
kubectl config use-context ap-southeast-1
kubectl apply -f k8s/networking/istio/virtualservice.yaml
kubectl apply -f k8s/networking/istio/gateway.yaml
```

### Step 5: Configure Prometheus Federation

Deploy federation scrape configs:

```bash
# Apply to each region
for context in us-east-1 eu-west-1 ap-southeast-1; do
  kubectl config use-context $context
  kubectl apply -f k8s/networking/federation/prometheus-federation.yaml
  kubectl apply -f k8s/networking/federation/multi-region-rules.yaml
done
```

### Step 6: Deploy Model Sync

```bash
# Deploy to primary region (US-EAST-1)
kubectl config use-context us-east-1
kubectl apply -f k8s/replication/model-sync.yaml

# Create GCP service account key secret
# First, create GCP service account
gcloud iam service-accounts create fingerprint-model-sync
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member=serviceAccount:fingerprint-model-sync@$PROJECT_ID.iam.gserviceaccount.com \
  --role roles/storage.admin

# Create and upload key
gcloud iam service-accounts keys create gcp-key.json \
  --iam-account=fingerprint-model-sync@$PROJECT_ID.iam.gserviceaccount.com

# Create secret in all regions
for context in us-east-1 eu-west-1 ap-southeast-1; do
  kubectl config use-context $context
  kubectl create secret generic gcp-service-account-key \
    --from-file=key.json=gcp-key.json \
    -n fingerprint-api
done
```

### Step 7: Deploy Cache Invalidation Watcher

```bash
# Deploy to all regions
for context in us-east-1 eu-west-1 ap-southeast-1; do
  kubectl config use-context $context
  kubectl apply -f k8s/replication/cache-invalidation.yaml
done
```

---

## ✅ Verification

### Verify Deployment Status

```bash
#!/bin/bash

echo "=== Multi-Region Deployment Status ==="
for region in us-east-1 eu-west-1 ap-southeast-1; do
  echo ""
  echo "Region: $region"
  kubectl config use-context $region
  kubectl get pods -n fingerprint-api
  kubectl get svc -n fingerprint-api
done
```

### Check Inter-Regional Connectivity

```bash
# From US-EAST pod, test EU-WEST connectivity
kubectl config use-context us-east-1
POD=$(kubectl get pods -n fingerprint-api -o jsonpath='{.items[0].metadata.name}')

# Test latency to EU-WEST
kubectl exec -it $POD -n fingerprint-api -- \
  curl -w "Time: %{time_total}s\n" \
  http://fingerprint-api-eu-west.fingerprint-api.svc.cluster.local:8000/status
```

### Monitor Prometheus Federation

```bash
# Port-forward Prometheus from US-EAST
kubectl config use-context us-east-1
kubectl port-forward svc/prometheus 9090:9090 -n monitoring &

# Access Prometheus UI
# http://localhost:9090

# Check for federation targets
# Targets page should show "fingerprint-api-federation-eu" and "fingerprint-api-federation-ap"
```

### Verify Model Sync

```bash
# Check CronJob status
kubectl config use-context us-east-1
kubectl get cronjobs -n fingerprint-api

# Check recent model sync jobs
kubectl get jobs -n fingerprint-api -l app=model-sync

# View model sync logs
kubectl logs -n fingerprint-api -l app=model-sync --tail=100
```

---

## 🌍 Global Traffic Routing

### Option 1: GeoDNS (Recommended)

```bash
# Using AWS Route53 (or equivalent)
# Create A record: api.fingerprint.example.com
#   - US region (50%): route53.us-east-1.fingerprint.example.com
#   - EU region (30%): route53.eu-west-1.fingerprint.example.com
#   - AP region (20%): route53.ap-southeast-1.fingerprint.example.com

# Geolocation routing rules:
# North America → US-EAST-1
# Europe → EU-WEST-1
# Asia-Pacific → AP-SOUTHEAST-1
# Default (other) → US-EAST-1 (primary)
```

### Option 2: Global Load Balancer

```bash
# Using Google Cloud Load Balancer (for GKE)
gcloud compute backend-services create fingerprint-api-global \
  --global \
  --protocol=HTTPS \
  --enable-cdn

# Add backends
gcloud compute backend-services add-backend fingerprint-api-global \
  --instance-group=fingerprint-api-us-east-ig \
  --instance-group-zone=us-east1-b \
  --global

# Create front-end IP and routing rules
gcloud compute addresses create fingerprint-api-global-ip --global
gcloud compute https-lb-create fingerprint-api-global \
  --address fingerprint-api-global-ip \
  --backend-service fingerprint-api-global
```

---

## 📊 Monitoring & Alerts

### Multi-Region Dashboard

Create a Grafana dashboard showing:

1. **Global Metrics**
   - Total request rate across regions
   - Global error rate
   - Global P99 latency

2. **Per-Region Metrics**
   - Requests per region
   - Error rate per region
   - Latency distribution (P50/P95/P99)

3. **Failover Status**
   - Pod count per region
   - Region availability percentage
   - Cross-region latency

### Multi-Region Alerts

```yaml
# Example alert rules
groups:
  - name: multi-region
    rules:
      - alert: RegionDown
        expr: up{job="fingerprint-api"} == 0
        for: 2m
        annotations:
          summary: "Fingerprint API region down"
          description: "Region {{ $labels.region }} has no healthy pods"

      - alert: CrossRegionLatencyHigh
        expr: histogram_quantile(0.99, http_request_duration_seconds_bucket{region=~"eu-.*"}) > 2
        annotations:
          summary: "Cross-region latency too high"
```

---

## 🔄 Failover Testing

### Simulate Region Failure

```bash
# Shutdown US-EAST pods
kubectl config use-context us-east-1
kubectl scale deployment/fingerprint-api --replicas=0 -n fingerprint-api

# Verify traffic routes to EU-WEST
# (When using circuit breaker + Istio)

# Check metrics in Prometheus
# Request count should transfer to EU-WEST

# Restore US-EAST
kubectl scale deployment/fingerprint-api --replicas=5 -n fingerprint-api
kubectl rollout status deployment/fingerprint-api -n fingerprint-api
```

### Load Test Failover

```bash
# Start load test from external client
while true; do
  curl -s api.fingerprint.example.com/status
  sleep 0.1
done

# Monitor:
# 1. Request latency (should increase when failing over)
# 2. Success rate (should remain 100%)
# 3. Region distribution (should shift to secondary/tertiary)
```

---

## 📈 Performance Targets

| Metric | Target | SLA |
|--------|--------|-----|
| **Latency (P99)** | <500ms to any region | 99.5% |
| **Region failover time** | <5 minutes | Critical |
| **Model sync time** | <30 minutes | Important |
| **Cross-region latency** | <2000ms | SLA |
| **Cache replication** | <1 minute | Important |

---

## 🚀 Next Steps

1. ✅ Complete Phase 9.1 deployment
2. 📋 Test multi-region failover scenarios
3. 📊 Configure multi-region monitoring
4. 🔄 Begin Phase 9.2 (Service Mesh Advanced Features)
5. 💾 Begin Phase 9.3 (Advanced Caching)

---

**Status**: Ready for deployment  
**Last Updated**: 2026-02-13
