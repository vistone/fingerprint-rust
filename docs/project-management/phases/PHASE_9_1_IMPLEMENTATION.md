# Phase 9.1: Multi-Region Deployment Implementation

**Status**: Starting  
**Estimated Duration**: 6 hours  
**Target Completion**: 90% of Phase 9.1  

---

## 🎯 Phase 9.1 Objectives

1. ✅ Design multi-region architecture with failover
2. ✅ Create regional cluster templates for US, EU, APAC
3. ✅ Implement cross-region service mesh networking
4. ✅ Set up model/cache replication
5. ✅ Configure Prometheus federation
6. ✅ Implement global traffic distribution

---

## 📊 Multi-Region Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Global Load Balancer                     │
│                    (GeoDNS / AWS Route53)                   │
└────────┬──────────────────────┬──────────────────────┬──────┘
         │                      │                      │
    ┌────v────┐            ┌───v────┐            ┌───v────┐
    │ US-EAST │            │ EU-WEST│            │ AP-SE   │
    │ Cluster │            │ Cluster │            │ Cluster │
    └────┬────┘            └───┬────┘            └───┬────┘
         │                     │                     │
    ┌────v─────────────────────v──────────────────────v────┐
    │         Distributed Service Mesh (Istio)             │
    │  • Cross-region traffic management                   │
    │  • Load balancing and failover                       │
    │  • Security policies across regions                  │
    └──────────────────────────────────────────────────────┘
         │                     │                     │
    ┌────v────┐            ┌───v────┐            ┌───v────┐
    │ PostgreSQL           │ Redis  │            │ S3/GCS │
    │ (Primary)            │ Cache  │            │Storage │
    │                      │ Cluster│            │        │
    └─────────────────────────────────────────────────────┘
         │                     │
    ┌────v──────────────────────v───┐
    │   Async Replication Layer      │
    │  • Model syncing (30min TTL)   │
    │  • Cache invalidation         │
    │  • Metrics federation          │
    └────────────────────────────────┘
```

---

## 🏗️ Regional Cluster Structure

### Cluster Specifications

| Region | Cluster | Nodes | Replicas | CPU/Memory | Purpose |
|--------|---------|-------|----------|-----------|---------|
| US-EAST | us-east-1 | 3-10 | 3-20 | 4CPU/8GB | Primary (low-latency NA) |
| EU-WEST | eu-west-1 | 3-8 | 3-15 | 4CPU/8GB | Secondary (low-latency EU) |
| AP-APAC | ap-southeast-1 | 2-5 | 2-10 | 2CPU/4GB | Tertiary (low-latency APAC) |

### Failure Scenarios

**Scenario 1: Region Down**
```
US-EAST unavailable
  ↓
GeoDNS routes US/NA traffic → EU-WEST (300ms+ latency)
  ↓
EU-WEST handles 3x load
  ↓
HPA scales to 15+ replicas
  ↓
Redis replication provides cache continuity
  ↓
Models cached locally (updated within 30 minutes)
```

**Scenario 2: Service Down (same region)**
```
US-EAST pods crash
  ↓
Kubernetes replaces pods (<60 seconds)
  ↓
Readiness probe detects unhealthy state
  ↓
Circuit breaker in Istio routes to EU-WEST
  ↓
Fallback accepted per SLA (different region, higher latency)
```

---

## 📁 Directory Structure

```
k8s/
├── base/                           # Current shared configs
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── ingress.yaml
│   ├── hpa.yaml
│   ├── rbac.yaml
│   ├── networkpolicy.yaml
│   ├── pdb.yaml
│   └── kustomization.yaml
│
├── overlays/
│   ├── production/                 # Current production (will become us-east-1)
│   ├── staging/
│   │
│   ├── us-east-1/                  # NEW: US East (Primary)
│   │   ├── kustomization.yaml      # Override replicas: 3-20
│   │   ├── deployment.yaml         # Add region label, affinity
│   │   └── values.yaml             # Region-specific config
│   │
│   ├── eu-west-1/                  # NEW: EU West (Secondary)
│   │   ├── kustomization.yaml      # Override replicas: 3-15
│   │   ├── deployment.yaml
│   │   └── values.yaml
│   │
│   └── ap-southeast-1/             # NEW: APAC (Tertiary)
│       ├── kustomization.yaml      # Override replicas: 2-10
│       ├── deployment.yaml
│       └── values.yaml
│
├── networking/                     # NEW: Cross-region networking
│   ├── istio/
│   │   ├── namespace.yaml          # Istio injection namespace
│   │   ├── virtualservice.yaml     # Traffic management
│   │   └── destinationrule.yaml    # Load balancing, failover
│   │
│   └── federation/
│       ├── prometheus-federation.yaml  # Prometheus scrape federation
│       └── grafana-datasources.yaml   # Multi-region data sources
│
└── replication/                    # NEW: Data replication
    ├── model-sync.yaml             # Model replication CronJob
    ├── cache-sync.yaml             # Cache invalidation
    └── metrics-aggregation.yaml    # Unified metrics collection
```

---

## 🔧 Implementation Tasks

### Task 1: Region-specific Kustomization Overlays

#### US-EAST-1 (Primary Region)

```yaml
# k8s/overlays/us-east-1/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: fingerprint-api

bases:
  - ../../base

commonLabels:
  region: us-east-1
  environment: production

patches:
  - target:
      kind: Deployment
      name: fingerprint-api
    patch: |-
      - op: replace
        path: /spec/replicas
        value: 5  # Start with 5, scale to 20 max
      - op: add
        path: /spec/template/spec/affinity
        value:
          podAntiAffinity:
            preferredDuringSchedulingIgnoredDuringExecution:
              - weight: 100
                podAffinityTerm:
                  labelSelector:
                    matchExpressions:
                      - key: app
                        operator: In
                        values:
                          - fingerprint-api
                  topologyKey: kubernetes.io/hostname
      - op: replace
        path: /spec/template/spec/containers/0/env
        value:
          - name: REGION
            value: us-east-1
          - name: PRIMARY_REGION
            value: "true"
          - name: LOG_LEVEL
            value: info

resources:
  - deployment.yaml
  - hpa.yaml

configMapGenerator:
  - name: region-config
    literals:
      - region=us-east-1
      - timezone=America/New_York
      - max_replicas=20
```

#### EU-WEST-1 (Secondary Region)

```yaml
# k8s/overlays/eu-west-1/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: fingerprint-api

bases:
  - ../../base

commonLabels:
  region: eu-west-1
  environment: production

patches:
  - target:
      kind: Deployment
      name: fingerprint-api
    patch: |-
      - op: replace
        path: /spec/replicas
        value: 3  # Can scale to 15
      - op: add
        path: /spec/template/spec/affinity
        value:
          podAntiAffinity:
            preferredDuringSchedulingIgnoredDuringExecution:
              - weight: 100
                podAffinityTerm:
                  labelSelector:
                    matchExpressions:
                      - key: app
                        operator: In
                        values:
                          - fingerprint-api
                  topologyKey: kubernetes.io/hostname
      - op: replace
        path: /spec/template/spec/containers/0/env
        value:
          - name: REGION
            value: eu-west-1
          - name: PRIMARY_REGION
            value: "false"
          - name: LOG_LEVEL
            value: info

configMapGenerator:
  - name: region-config
    literals:
      - region=eu-west-1
      - timezone=Europe/London
      - max_replicas=15
```

### Task 2: Istio Service Mesh Configuration

```yaml
# k8s/networking/istio/virtualservice.yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: fingerprint-api
  namespace: fingerprint-api
spec:
  hosts:
  - fingerprint-api
  http:
  # Route to local region with preference
  - name: local-region
    match:
    - uri:
        prefix: /
    route:
    - destination:
        host: fingerprint-api
        subset: local-region
      weight: 95
    # Fallback to EU-WEST if local unavailable
    - destination:
        host: fingerprint-api-eu
        subset: eu-west
      weight: 5
    timeout: 2s
    retries:
      attempts: 3
      perTryTimeout: 1s

---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: fingerprint-api
  namespace: fingerprint-api
spec:
  host: fingerprint-api
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        http1MaxPendingRequests: 100
        http2MaxRequests: 100
        maxRequestsPerConnection: 2
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 100
  subsets:
  - name: local-region
    labels:
      region: us-east-1
  - name: eu-west
    labels:
      region: eu-west-1
```

### Task 3: Cross-Region Data Replication

```yaml
# k8s/replication/model-sync.yaml - Sync ML models between regions
apiVersion: batch/v1
kind: CronJob
metadata:
  name: model-sync
  namespace: fingerprint-api
spec:
  schedule: "*/15 * * * *"  # Every 15 minutes
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: model-sync
          containers:
          - name: model-sync
            image: fingerprint-api:latest
            command:
            - /app/scripts/sync-models.sh
            env:
            - name: SOURCE_BUCKET
              value: gs://fingerprint-models-us-east
            - name: DEST_BUCKET
              value: gs://fingerprint-models-eu-west
            - name: DEST_BUCKET_2
              value: gs://fingerprint-models-ap-southeast
            - name: SYNC_TIMEOUT
              value: "600"
            resources:
              requests:
                memory: "512Mi"
                cpu: "500m"
              limits:
                memory: "1Gi"
                cpu: "1000m"
          restartPolicy: OnFailure
          backoffLimit: 3
```

### Task 4: Prometheus Federation for Multi-Region

```yaml
# monitoring/prometheus/federation-config.yaml
# This runs in each region's Prometheus to aggregate metrics
global:
  scrape_interval: 30s
  evaluation_interval: 30s
  external_labels:
    cluster: fingerprint-api
    region: us-east-1  # Change per region

scrape_configs:
  # Local region metrics
  - job_name: 'fingerprint-api-local'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - fingerprint-api

  # Remote region metrics (federation)
  - job_name: 'fingerprint-api-eu'
    params:
      'match[]':
        - '{job="fingerprint-api"}'
    static_configs:
      - targets:
          - 'prometheus-eu.example.com:9090/federate'
    honor_labels: true
    metric_relabel_configs:
      - source_labels: [__name__]
        regex: 'up|http_requests_total|http_request_duration.*'
        action: keep

  - job_name: 'fingerprint-api-ap'
    params:
      'match[]':
        - '{job="fingerprint-api"}'
    static_configs:
      - targets:
          - 'prometheus-ap.example.com:9090/federate'
    honor_labels: true
    metric_relabel_configs:
      - source_labels: [__name__]
        regex: 'up|http_requests_total|http_request_duration.*'
        action: keep

# Aggregation rules for multi-region metrics
rule_files:
  - /etc/prometheus/multi-region-rules.yaml
```

### Task 5: Global Load Balancer Configuration

```yaml
# Global load balancer using Istio Gateway + Service

---
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: fingerprint-api-global
  namespace: fingerprint-api
spec:
  selector:
    istio: ingressgateway
  servers:
  - port:
      number: 443
      name: https
      protocol: HTTPS
    tls:
      mode: SIMPLE
      credentialName: fingerprint-api-cert
    hosts:
    - "api.fingerprint.example.com"
  - port:
      number: 80
      name: http
      protocol: HTTP
    hosts:
    - "api.fingerprint.example.com"

---
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: fingerprint-api-global
  namespace: fingerprint-api
spec:
  hosts:
  - api.fingerprint.example.com
  gateways:
  - fingerprint-api-global
  http:
  - name: api-route
    route:
    - destination:
        host: fingerprint-api-us-east
        port:
          number: 8000
      weight: 50
    - destination:
        host: fingerprint-api-eu-west
        port:
          number: 8000
      weight: 30
    - destination:
        host: fingerprint-api-ap-southeast
        port:
          number: 8000
      weight: 20
    timeout: 3s
    retries:
      attempts: 3
      perTryTimeout: 1s
```

---

## 📋 Deployment Checklist

### Pre-Deployment
- [ ] Review multi-region architecture diagram
- [ ] Verify regional cluster connectivity
- [ ] Test cross-region network latency
- [ ] Verify DNS setup for GeoDNS
- [ ] Check TLS certificate for multi-region domain
- [ ] Validate model storage replication setup
- [ ] Verify Redis cluster across regions
- [ ] Test metric federation setup

### Deployment Steps
```bash
# 1. Deploy US-EAST (Primary)
kubectl apply -k k8s/overlays/us-east-1/

# 2. Verify US-EAST deployment
kubectl rollout status deployment/fingerprint-api -n fingerprint-api --kubeconfig=us-east-1.conf

# 3. Deploy EU-WEST (Secondary)
kubectl apply -k k8s/overlays/eu-west-1/ --kubeconfig=eu-west-1.conf

# 4. Deploy AP-SOUTHEAST (Tertiary)
kubectl apply -k k8s/overlays/ap-southeast-1/ --kubeconfig=ap-southeast-1.conf

# 5. Deploy Istio Federation
for region in us-east-1 eu-west-1 ap-southeast-1; do
  kubectl apply -f k8s/networking/istio/virtualservice.yaml --kubeconfig=$region.conf
done

# 6. Deploy Model Sync
kubectl apply -f k8s/replication/model-sync.yaml --kubeconfig=us-east-1.conf

# 7. Verify all regions
./scripts/verify-multi-region.sh
```

### Post-Deployment
- [ ] Verify API availability in all regions
- [ ] Test latency from each region
- [ ] Verify model replication
- [ ] Check Prometheus federation
- [ ] Test failover scenarios
- [ ] Verify SLA targets met
- [ ] Load test cross-region failover
- [ ] Update documentation

---

## ✅ Success Criteria

| Metric | Target | Status |
|--------|--------|--------|
| US-EAST latency (P99) | <100ms | ⏳ |
| EU-WEST latency (P99) | <150ms | ⏳ |
| AP latency (P99) | <200ms | ⏳ |
| Cross-region failover | <5min | ⏳ |
| Model sync latency | <30min | ⏳ |
| Prometheus federation | <60s | ⏳ |
| API availability | 99.5% | ⏳ |

---

## 📝 Next Steps

1. Create regional cluster templates
2. Set up Istio service mesh
3. Configure model replication
4. Deploy Prometheus federation
5. Test multi-region failover
6. Document operational procedures
7. Create multi-region deployment guide

---

**Status**: Ready to implement  
**Estimated Completion**: 6 hours  
**Next Milestone**: Phase 9.1 complete, then Phase 9.2 (Service Mesh)
