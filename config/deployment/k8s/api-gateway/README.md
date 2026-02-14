# API Gateway & Distributed Rate Limiting

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---



This directory contains Kubernetes manifests for deploying Kong API Gateway with Redis-backed distributed rate limiting.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Users                                │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  Kong API Gateway (3 replicas)                              │
│  ├─ SSL/TLS Termination                                     │
│  ├─ Request Routing                                         │
│  ├─ Rate Limiting (Redis-backed)                           │
│  ├─ Authentication (Key Auth / JWT)                        │
│  └─ CORS Handling                                          │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┴───────────────┐
        │                               │
        ▼                               ▼
┌───────────────┐           ┌───────────────────┐
│  PostgreSQL   │           │  Redis Cluster    │
│  (Kong DB)    │           │  (Rate Limiting)  │
└───────────────┘           └───────────────────┘
                                        │
                        ┌───────────────┴───────────────┐
                        │                               │
                        ▼                               ▼
            ┌───────────────────┐           ┌───────────────────┐
            │  fingerprint-api  │           │  Rate Limit Sync  │
            │  (Backend)        │           │  (Rust)           │
            └───────────────────┘           └───────────────────┘
```

## Components

| Component | Purpose | Replicas |
|-----------|---------|----------|
| Kong Gateway | API Gateway, routing, rate limiting | 3 (dev: 1) |
| PostgreSQL | Kong configuration database | 1 (external in prod) |
| Redis | Distributed rate limiting store | 3 (dev: 1) |

## Quick Start

### Prerequisites

- Kubernetes cluster (1.24+)
- kubectl configured
- kustomize (optional, kubectl built-in works)

### Deploy to Development

```bash
# Deploy all components
./deploy.sh -e development

# Or manually with kubectl
kubectl apply -k overlays/development

# Configure Kong routes and plugins
./scripts/configure-kong.sh -n api-gateway-dev
```

### Deploy to Production

```bash
# Deploy with production overlays
./deploy.sh -e production

# Configure Kong
./scripts/configure-kong.sh -n api-gateway
```

### Delete Deployment

```bash
./deploy.sh -e development -d
```

## File Structure

```
api-gateway/
├── deploy.sh                    # Main deployment script
├── kustomization.yaml           # Base Kustomize configuration
├── README.md                    # This file
│
├── kong-deployment.yaml         # Kong gateway deployment
├── kong-postgres.yaml          # PostgreSQL for Kong
├── kong-plugins.yaml           # Kong plugins configuration
├── rate-limiting-configmap.yaml # Rate limiting quotas & policies
├── redis-deployment.yaml       # Redis cluster for rate limiting
│
├── overlays/
│   ├── development/
│   │   └── kustomization.yaml  # Dev environment (lightweight)
│   ├── staging/
│   │   └── kustomization.yaml  # Staging environment
│   └── production/
│       └── kustomization.yaml  # Production environment (HA)
│
└── scripts/
    └── configure-kong.sh       # Kong configuration script
```

## Rate Limiting Configuration

### User Tiers

| Tier | Requests/Min | Requests/Hour | Requests/Month | Features |
|------|-------------|---------------|----------------|----------|
| Free | 100 | 3,000 | 50,000 | Basic fingerprinting |
| Pro | 1,000 | 30,000 | 1,000,000 | Advanced features |
| Enterprise | 10,000 | Unlimited | Unlimited | SLA, custom models |

### Endpoint Costs

| Endpoint | Cost | Rate Limit Multiplier |
|----------|------|---------------------|
| `/fingerprint/identify` | 1 token | 1.0x |
| `/fingerprint/compare` | 2 tokens | 0.5x |
| `/fingerprint/batch` | 1 per item | 0.3x |
| `/health` | 0 | No limit |

## Accessing the Services

### Port Forwarding

```bash
# Gateway (HTTP)
kubectl port-forward svc/kong-gateway 8080:80 -n api-gateway

# Admin API
kubectl port-forward svc/kong-admin 8001:8001 -n api-gateway

# Redis (for debugging)
kubectl port-forward svc/redis-cluster 6379:6379 -n caching
```

### Testing

```bash
# Health check (no auth)
curl http://localhost:8080/health

# With API key
curl -H "apikey: YOUR_API_KEY" http://localhost:8080/fingerprint/identify

# Check rate limit headers
curl -i -H "apikey: YOUR_API_KEY" http://localhost:8080/fingerprint/identify
```

## Monitoring

### Prometheus Metrics

Kong exposes metrics at `:8100/metrics`:

```bash
# Get metrics
curl http://localhost:8100/metrics
```

Metrics include:
- `kong_http_status` - HTTP status codes
- `kong_latency` - Request latency
- `kong_bandwidth` - Request/response bytes

### Redis Metrics

Redis exporter exposes metrics at `:9121/metrics`:

```bash
# Port forward to exporter
kubectl port-forward svc/redis-exporter 9121:9121 -n caching

# Get metrics
curl http://localhost:9121/metrics
```

## Troubleshooting

### Kong not starting

```bash
# Check migrations job
kubectl logs -n api-gateway job/kong-migrations

# Check Kong logs
kubectl logs -n api-gateway deployment/kong
```

### Rate limiting not working

1. Check Redis connectivity:
   ```bash
   kubectl exec -it redis-0 -n caching -- redis-cli ping
   ```

2. Verify rate-limiting plugin:
   ```bash
   curl http://localhost:8001/plugins
   ```

### High memory usage

Redis memory is limited to 512MB per pod with `allkeys-lru` eviction policy.

## Security Considerations

1. **Change default passwords** in production
2. **Use external PostgreSQL** for high availability
3. **Enable TLS** for external traffic
4. **Restrict Admin API** access (127.0.0.1 only in prod)
5. **Use network policies** to restrict pod-to-pod traffic

## References

- [Kong Documentation](https://docs.konghq.com/)
- [Kong Rate Limiting Plugin](https://docs.konghq.com/hub/kong-inc/rate-limiting/)
- [Kustomize Documentation](https://kubectl.docs.kubernetes.io/references/kustomize/)
