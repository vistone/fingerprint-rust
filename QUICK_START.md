# 🚀 Fingerprint API - Quick Start Guide

**Status**: ✅ Production Ready (Phase 8 Complete, 85% Overall)

## 📋 Quick Links

| Need | File | Purpose |
|------|------|---------|
| **Deploy** | [PHASE_8_DEPLOYMENT_GUIDE.md](docs/PHASE_8_DEPLOYMENT_GUIDE.md) | Step-by-step Kubernetes deployment |
| **Operate** | [OPERATIONS_RUNBOOK.md](docs/OPERATIONS_RUNBOOK.md) | Daily ops, incident response, scaling |
| **Monitor** | [SLA_AND_MONITORING_PROTOCOL.md](docs/SLA_AND_MONITORING_PROTOCOL.md) | SLOs, alerting, monitoring protocols |
| **Troubleshoot** | [TROUBLESHOOTING_GUIDE.md](docs/TROUBLESHOOTING_GUIDE.md) | Issue diagnosis and resolution |
| **Architecture** | [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design and component overview |
| **REST API** | [API.md](docs/API.md) | API endpoints and usage examples |

---

## 🎯 5-Minute Setup

```bash
# 1. Clone the repository
git clone <repo-url>
cd fingerprint-rust

# 2. Customize for your environment
cd k8s/overlays/production
# Edit:
#   - kustomization.yaml (image registry, domain)
#   - deployment.yaml (replicas, resources)
# Edit ../../../monitoring/prometheus/prometheus.yml (your services)

# 3. Deploy to Kubernetes
./k8s/deploy.sh

# 4. Verify deployment
kubectl get pods -n fingerprint-api
kubectl logs -n fingerprint-api -l app=fingerprint-api -f
```

---

## 📊 What You Get

### ✅ Core Service
- Production-ready REST API (5 endpoints)
- Browser fingerprinting with TLS, HTTP headers, DNS, profiles
- Machine learning inference (18 trained models)
- <1s P99 latency, 99.5% availability

### ✅ Infrastructure
- **Kubernetes**: HA deployment with auto-scaling (3-20 replicas)
- **Monitoring**: Prometheus + Grafana with 22 alert rules
- **Logging**: ELK stack with intelligent HTTP request parsing
- **Security**: RBAC, NetworkPolicy, non-root execution, read-only FS

### ✅ Operations
- Daily operational checklists
- 5 incident response runbooks (API down, high error, latency, crashes, disk full)
- Backup and disaster recovery procedures
- Performance tuning and scaling guides
- 24/7 monitoring with SLA definition (99.5% availability)

---

## 🔧 Daily Operations

### Check System Health
```bash
# Pod status
kubectl get pods -n fingerprint-api

# API health
curl http://localhost:8000/health

# View recent logs
kubectl logs -n fingerprint-api -l app=fingerprint-api --tail=100

# Check Prometheus metrics
# Access: http://localhost:9090
```

### Common Tasks

**Scale up for traffic spike:**
```bash
kubectl scale deployment -n fingerprint-api fingerprint-api --replicas=10
```

**View API error logs in Kibana:**
```bash
# Go to http://localhost:5601
# Search: severity:ERROR
```

**Check cluster metrics:**
```bash
# Grafana dashboard: http://localhost:3000
# Username: admin, Password: admin
```

---

## 📞 Emergency Response

### API Service Down
1. Check pod status: `kubectl get pods -n fingerprint-api`
2. Check logs: `kubectl logs -n fingerprint-api <pod-name>`
3. Verify service: `kubectl get svc -n fingerprint-api`
4. See full runbook: [OPERATIONS_RUNBOOK.md](docs/OPERATIONS_RUNBOOK.md#incident-api-service-down)

### High Error Rate
1. Check error logs: `kubectl logs -n fingerprint-api -l app=fingerprint-api | grep ERROR`
2. Check Prometheus: Look for error spike in Grafana
3. Scale if needed: `kubectl scale deployment -n fingerprint-api fingerprint-api --replicas=15`
4. See full runbook: [OPERATIONS_RUNBOOK.md](docs/OPERATIONS_RUNBOOK.md#incident-high-error-rate)

### Performance Issues
1. Check latency: View Prometheus `http_request_duration_seconds_p99`
2. Check CPU/Memory: `kubectl top pods -n fingerprint-api`
3. Review slowlog in Kibana: Search for request_duration > 1000ms
4. See full guide: [OPERATIONS_RUNBOOK.md](docs/OPERATIONS_RUNBOOK.md#performance-tuning)

---

## 📈 Monitoring Dashboard

**Grafana Dashboards**:
- Main operational dashboard (8 panels)
- Access: `http://kubernetes-node:30030`
- Username: admin
- Password: admin

**Key Metrics**:
- Request rate (req/sec)
- P50/P95/P99 latency
- Error rate (%)
- Pod memory/CPU usage
- Model inference latency

---

## 🔒 Security

**Enabled protections**:
- Non-root user execution (UID 1000)
- Read-only root filesystem
- Network policies (ingress/egress controlled)
- RBAC with least privilege
- Resource limits enforced

**Before production**: Update TLS certificates and registry credentials in overlays.

---

## 📋 SLA & Support

**Service Level Objectives**:
- Availability: 99.5% (216 minutes error budget/month)
- P99 Latency: <1 second
- Error Rate: <0.1%
- Data Accuracy: ≥95% browser classification

**Alert Thresholds**:
- 🔴 Critical: API down, error >5%, disk full
- 🟠 High: Latency >1s, pod crashes, CPU >80%
- 🟡 Warning: Disk >80%, frequent restarts

See [SLA_AND_MONITORING_PROTOCOL.md](docs/SLA_AND_MONITORING_PROTOCOL.md) for details.

---

## 📚 Documentation Structure

```
docs/
├── API.md                           # REST API endpoints
├── ARCHITECTURE.md                  # System design
├── OPERATIONS_RUNBOOK.md            # Daily & incident procedures (1200+ lines)
├── SLA_AND_MONITORING_PROTOCOL.md  # SLOs and alerting (600+ lines)
├── TROUBLESHOOTING_GUIDE.md        # Issue diagnosis (800+ lines)
├── PHASE_8_DEPLOYMENT_GUIDE.md     # Kubernetes deployment
└── guides/                          # Additional implementation guides
```

---

## 🚀 Next Steps

1. **Deploy**: Follow [PHASE_8_DEPLOYMENT_GUIDE.md](docs/PHASE_8_DEPLOYMENT_GUIDE.md)
2. **Monitor**: Set up Grafana dashboards in your cluster
3. **Operate**: Use [OPERATIONS_RUNBOOK.md](docs/OPERATIONS_RUNBOOK.md) for daily tasks
4. **Scale**: Configure HPA in k8s/base/hpa.yaml for your traffic patterns

---

## 💾 Project Status

| Phase | Component | Status | Status |
|-------|-----------|--------|--------|
| 1-6 | Fingerprinting foundations | ✅ Complete | 25% |
| 7 | ML training + REST API | ✅ Complete | 35% |
| 8 | Kubernetes + Monitoring + Ops | ✅ Complete | 25% |
| 9-10 | Advanced features (future) | 📋 Planned | 15% |

**Overall Progress**: ✅ **85% Complete**

---

## 📞 Support

- **Deployment issues**: See deployment guide
- **Operational questions**: See runbook
- **Troubleshooting**: See troubleshooting guide
- **Performance tuning**: See performance section in runbook
- **SLA/monitoring**: See SLA documentation

---

**Last Updated**: 2026-02-13  
**Phase**: 8 Production Deployment (Complete)  
**Status**: ✅ Production Ready
