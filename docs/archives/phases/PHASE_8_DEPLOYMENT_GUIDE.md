# Phase 8 Production Deployment Guide

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



## Overview

Phase 8 provides a complete production-ready infrastructure for deploying the Fingerprint API on Kubernetes with enterprise-grade monitoring, logging, and auto-scaling capabilities.

## Quick Start

### Prerequisites

- Kubernetes cluster (1.20+) with `kubectl` configured
- `kustomize` installed (`kubectl` includes it, or install separately)
- Helm (optional, for cert-manager and other operators)
- A DNS domain for Ingress (e.g., `fingerprint-api.example.com`)

### Staging Deployment (Recommended First Step)

```bash
# 1. Preview the manifests that will be deployed
./deploy.sh staging preview

# 2. Dry-run to validate (recommended)
./deploy.sh staging apply --dry-run

# 3. Deploy to staging when ready
./deploy.sh staging apply

# 4. Monitor deployment progress
kubectl rollout status deployment/fingerprint-api -n fingerprint
kubectl get pods -n fingerprint -w

# 5. Access the API
kubectl port-forward -n fingerprint svc/fingerprint-api 8080:80
# API now available at http://localhost:8080
```

### Production Deployment

```bash
# 1. Ensure staging is working and validated
kubectl get pods -n fingerprint

# 2. Preview production manifests (5 replicas, resource limits)
./deploy.sh production preview

# 3. Validate production configuration
./deploy.sh production apply --dry-run

# 4. Deploy to production
./deploy.sh production apply

# 5. Verify all services are running
kubectl get all -n fingerprint
kubectl get hpa -n fingerprint
```

## Architecture

### Components

#### Fingerprint API Service
- **Namespace**: `fingerprint`
- **Deployment**: `fingerprint-api`
- **Replicas**: 3 (staging), 5 (production), auto-scales 3-20 with HPA
- **Service**: `fingerprint-api` (ClusterIP)
- **Ingress**: `fingerprint-api-ingress` (HTTPS via cert-manager)

#### Monitoring Stack
- **Prometheus** (namespace: `prometheus`)
  - 2 replicas
  - 15-day data retention
  - Kubernetes service discovery
  
- **Alertmanager** (namespace: `prometheus`)
  - Multi-channel routing (Slack, PagerDuty, Email)
  - Critical alert escalation
  
- **Grafana** (integrated with Prometheus)
  - Main API overview dashboard
  - Extensible dashboard system

#### Logging Stack
- **Elasticsearch** (namespace: `logging`)
  - Centralized log storage
  - Daily indices: `fingerprint-api-YYYY.MM.dd`
  
- **Logstash** (namespace: `logging`)
  - Log ingestion from Kubernetes pods
  - HTTP request parsing
  - Error classification and alerting
  
- **Kibana** (namespace: `logging`)
  - Log visualization and search

### Scalability Features

#### Horizontal Pod Autoscaler (HPA)
- **Min Replicas**: 3 (staging), 5 (production)
- **Max Replicas**: 10 (staging), 20 (production)
- **Metrics**:
  - CPU utilization: 70% threshold
  - Memory utilization: 80% threshold
  - Custom metrics: 100 requests/second per pod

#### Resource Requests & Limits
```
Requests:  500m CPU, 512Mi memory
Limits:   2000m CPU,  2Gi  memory
```

### High Availability

- **Replica Distribution**: Pod anti-affinity ensures pods spread across nodes
- **Pod Disruption Budget**: Maintains minimum 2 pods during cluster disruptions
- **Health Probes**:
  - Liveness: `/health` (30s delay, 10s period)
  - Readiness: `/api/v1/models/status` (10s delay, 5s period)
- **Rolling Updates**: MaxSurge=1, MaxUnavailable=0 (zero-downtime deployments)

### Security

#### Container Security
- **Non-root user** (UID 1000)
- **Read-only root filesystem** (except /tmp)
- **Dropped capabilities** (NET_RAW, SYS_CHROOT, etc.)

#### Network Security
- **NetworkPolicy**: Restricts ingress/egress traffic
  - Ingress: Only from Ingress controller and Prometheus
  - Egress: DNS, HTTPS, HTTP, and internal services
  
#### RBAC
- **ServiceAccount**: `fingerprint-api`
- **Permissions**: ConfigMap, Secret, Pod, Event reading only
- **ClusterRole**: Kubernetes metrics API access for HPA

## Directory Structure

```
k8s/
├── base/                          # Base Kubernetes manifests
│   ├── namespace.yaml             # 'fingerprint' namespace
│   ├── configmap.yaml             # Application configuration
│   ├── deployment.yaml            # Main API deployment (3 replicas)
│   ├── service.yaml               # ClusterIP service
│   ├── ingress.yaml               # HTTPS routing (cert-manager)
│   ├── hpa.yaml                   # Auto-scaling (3-10 replicas)
│   ├── rbac.yaml                  # ServiceAccount, Role, RoleBinding
│   ├── networkpolicy.yaml         # Network access controls
│   ├── pdb.yaml                   # Pod disruption budget
│   └── kustomization.yaml         # Manifest aggregation

overlays/
├── staging/                       # Staging-specific overrides
│   ├── kustomization.yaml         # 2 replicas, debug logging
│   └── deployment.yaml            # Reduced resource limits
│
└── production/                    # Production-specific overrides
    ├── kustomization.yaml         # 5 replicas, warning logging
    ├── deployment.yaml            # Full resource limits
    └── hpa.yaml                   # Extended scaling range (5-20)

monitoring/
├── prometheus/
│   ├── prometheus.yml             # Scrape configurations
│   ├── alert_rules.yml            # Critical alert rules (10 total)
│   ├── recording_rules.yml        # Pre-computed metrics (16)
│   ├── alertmanager.yml           # Alert routing
│   └── prometheus-deployment.yaml # K8s deployment manifest
│
├── grafana/
│   ├── datasources/
│   │   └── prometheus-datasource.yaml
│   └── dashboards/
│       └── fingerprint-api-overview.json
│
└── elk/
    ├── elasticsearch/
    │   └── elasticsearch.yml      # ES configuration
    ├── logstash/
    │   └── fingerprint-api.conf   # Log processing pipeline
    └── kibana/
        └── kibana.yml             # Kibana configuration
```

## Key Configuration Files

### Deployment Configuration

**Resource Limits** (deployment.yaml):
```yaml
requests:
  cpu: 500m
  memory: 512Mi
limits:
  cpu: 2000m
  memory: 2Gi
```

**Health Probes** (deployment.yaml):
```yaml
livenessProbe:
  path: /health
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  path: /api/v1/models/status
  initialDelaySeconds: 10
  periodSeconds: 5
```

### Application Configuration (configmap.yaml)

```yaml
log_level: "info"
workers: "4"
max_pool_size: "100"
request_timeout: "30"
feature_extraction_timeout: "5"
inference_timeout: "10"
enable_cors: "true"
enable_metrics: "true"
```

### Alert Rules (alert_rules.yml)

- **FingerPrintAPIDown**: API unreachable >2m
- **HighErrorRate**: >5% error rate
- **HighLatency**: P99 latency >1s
- **PodCrashLooping**: >5 restarts/hour
- **HighCPUUsage**: >80% CPU
- **HighMemoryUsage**: >85% of limit
- **ModelLoadingFailure**: Models unavailable
- **HighQueueLatency**: Queue wait >5s

## Monitoring & Observability

### Prometheus Metrics

Access Prometheus UI (if port-forwarded):
```bash
kubectl port-forward -n prometheus svc/prometheus 9090:9090
# http://localhost:9090
```

Key metrics:
- `http_requests_total` - Total API requests
- `http_request_duration_seconds` - Request latency histogram
- `fingerprint_api_inference_duration_seconds` - Model inference latency
- `container_cpu_usage_seconds_total` - Pod CPU usage
- `container_memory_usage_bytes` - Pod memory usage

### Grafana Dashboards

Access Grafana (if port-forwarded):
```bash
kubectl port-forward -n grafana svc/grafana 3000:3000
# http://localhost:3000 (default: admin/admin)
```

Available dashboards:
- **Fingerprint API Overview** - 8-panel real-time monitoring
- Build custom dashboards by querying Prometheus

### Elasticsearch & Kibana

Access Kibana (if port-forwarded):
```bash
kubectl port-forward -n logging svc/kibana 5601:5601
# http://localhost:5601
```

Log indices:
- `fingerprint-api-2026.02.13` (daily rotation)

Searches:
- Log level: `severity:ERROR`
- Endpoint: `endpoint:"/api/v1/fingerprint/identify"`
- Response time: `response_time_ms:[1000 TO *]`

## Troubleshooting

### Deployment Issues

**Pods not starting**:
```bash
# Check pod status
kubectl get pods -n fingerprint

# View pod events
kubectl describe pod <pod-name> -n fingerprint

# Check logs
kubectl logs <pod-name> -n fingerprint
```

**Readiness probe failing**:
```bash
# Test endpoint manually
kubectl exec -it <pod-name> -n fingerprint -- curl http://localhost:8000/api/v1/models/status
```

**HPA not scaling**:
```bash
# Check HPA status
kubectl get hpa -n fingerprint
kubectl describe hpa fingerprint-api-hpa -n fingerprint

# Check metrics server (required for HPA)
kubectl get deployment metrics-server -n kube-system
```

### Configuration Issues

**ConfigMap not updating**:
```bash
# Restart deployment to pick up new config
kubectl rollout restart deployment/fingerprint-api -n fingerprint
```

**Ingress HTTPS not working**:
```bash
# Check cert-manager status
kubectl get certificate -n fingerprint
kubectl describe certificate fingerprint-api-tls -n fingerprint

# View certificate details
kubectl get secret fingerprint-api-tls -n fingerprint -o yaml
```

### Monitoring Issues

**Prometheus not scraping metrics**:
```bash
# Check prometheus config
kubectl get cm prometheus-config -n prometheus -o yaml

# Check targets in Prometheus UI
# http://localhost:9090/targets
```

**Alerts not firing**:
```bash
# Check alertmanager config
kubectl get cm prometheus-config -n prometheus -o yaml

# View alertmanager UI (port-forward :9093)
kubectl port-forward -n prometheus svc/alertmanager 9093:9093
```

## Production Checklist

Before going live in production:

- [ ] Staging deployment validated for 24+ hours
- [ ] All health probes passing
- [ ] HPA scaling tested under load
- [ ] Monitoring and alerting configured
- [ ] Backup and disaster recovery tested
- [ ] RBAC and network policies reviewed
- [ ] DNS record points to Ingress controller
- [ ] SSL certificate issued and valid
- [ ] Log retention policy configured
- [ ] OnCall rotation for alerts configured

## Scaling Considerations

### Horizontal Scaling

The HPA automatically adjusts pod count based on metrics. To manually scale:

```bash
# Scale staging to 4 replicas
kubectl scale deployment fingerprint-api -n fingerprint --replicas=4
```

### Vertical Scaling

To increase resource limits:

```bash
# Edit deployment
kubectl edit deployment fingerprint-api -n fingerprint

# Update resource requests/limits, then save
```

### Database/Model Considerations

- Ensure model files are accessible to all pods (via ConfigMap or shared storage)
- Monitor model loading latency
- Plan for periodic model updates without downtime

## Disaster Recovery

### Backup

```bash
# Backup current state
kubectl get all -n fingerprint -o yaml > backup-$(date +%Y%m%d).yaml

# Backup Persistent Volumes (if used)
kubectl get pv -o yaml > pv-backup-$(date +%Y%m%d).yaml
```

### Restore

```bash
# Delete existing resources
./deploy.sh production delete

# Restore from backup
kubectl apply -f backup-2026020113.yaml
```

## Support & Documentation

- **Phase 8 Implementation Report**: `docs/PHASE_8_IMPLEMENTATION_REPORT.md`
- **Planning Document**: `docs/PHASE_8_PRODUCTION_DEPLOYMENT_PLAN.md`
- **REST API Documentation**: `phase7_api/README.md`
- **Kubernetes Official Docs**: https://kubernetes.io/docs/
- **Prometheus Guide**: https://prometheus.io/docs/

---

**Phase 8 Deployment Status**: ✅ Ready for Production  
**Project Progress**: 82% Complete (Phases 1-8 infrastructure ready)
