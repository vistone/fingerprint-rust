# Phase 9.2: Service Mesh Advanced Features (Istio)

**Status**: Starting  
**Estimated Duration**: 5-7 hours  
**Target Completion**: 90% of Phase 9.2

---

## 🎯 Phase 9.2 Objectives

1. ✅ Advanced traffic management (canary deployments, traffic splitting)
2. ✅ Distributed tracing setup (Jaeger integration)
3. ✅ Rate limiting and quota management per client
4. ✅ Service mesh observability enhancements
5. ✅ Kiali dashboards for mesh visualization
6. ✅ Security policies (AuthorizationPolicy)

---

## 🏗️ Service Mesh Architecture Update

### Before (Phase 9.1)
```
VirtualService → DestinationRule → Pods (static routing)
```

### After (Phase 9.2)
```
VirtualService → (traffic splitting: 95/5 for canary)
    → Circuit Breaker → Outlier Detection
    → Rate Limiting → Quota Manager
    → Distributed Tracing → Jaeger Backend
    → Observability → Kiali Dashboards
    → AuthorizationPolicy → mTLS enforcement
```

---

## 📋 Implementation Tasks

### Task 1: Advanced Traffic Management

#### 1.1 Canary Deployment Strategy

```yaml
# k8s/networking/istio/canary-deployment.yaml
---
# Canary VirtualService for safe deployments
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: fingerprint-api-canary
  namespace: fingerprint-api
spec:
  hosts:
  - fingerprint-api.fingerprint-api.svc.cluster.local
  http:
  # Canary traffic management (95% stable, 5% canary)
  - name: canary-route
    match:
    - headers:
        x-user-id:
          regex: "^canary-.*"
    route:
    - destination:
        host: fingerprint-api.fingerprint-api.svc.cluster.local
        subset: canary
      weight: 5
    - destination:
        host: fingerprint-api.fingerprint-api.svc.cluster.local
        subset: stable
      weight: 95
    timeout: 3s
    retries:
      attempts: 3
      perTryTimeout: 1s
  
  # Regular traffic (all stable)
  - name: stable-route
    route:
    - destination:
        host: fingerprint-api.fingerprint-api.svc.cluster.local
        subset: stable
      weight: 100
    timeout: 3s
    retries:
      attempts: 3
      perTryTimeout: 1s

---
# DestinationRule with canary subsets
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: fingerprint-api-canary
  namespace: fingerprint-api
spec:
  host: fingerprint-api.fingerprint-api.svc.cluster.local
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 200
      http:
        http1MaxPendingRequests: 150
        http2MaxRequests: 150
        maxRequestsPerConnection: 2
    loadBalancer:
      consistentHash:
        httpHeaderName: x-user-id
    outlierDetection:
      consecutive5xxErrors: 3
      interval: 10s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
      minRequestVolume: 50
      splitExternalLocalOriginErrors: true
  subsets:
  - name: stable
    labels:
      version: stable
  - name: canary
    labels:
      version: canary
```

#### 1.2 Traffic Shifting Implementation

```yaml
# k8s/networking/istio/traffic-shifting.yaml
---
# Progressive traffic shift (automated rollout)
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: fingerprint-api
  namespace: fingerprint-api
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: fingerprint-api
  progressDeadlineSeconds: 300
  service:
    port: 8000
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
    - name: request-success-rate
      thresholdRange:
        min: 99
      interval: 1m
    - name: request-duration
      thresholdRange:
        max: 500  # ms
      interval: 1m
  webhooks:
  - name: acceptance-test
    url: http://flagger-loadtester/
    timeout: 5s
    metadata:
      type: smoke
      cmd: "curl -sd 'test' http://fingerprint-api-canary/status | grep ok"
  - name: load-test
    url: http://flagger-loadtester/
    timeout: 5s
    metadata:
      type: bash
      cmd: "ab -n 100 -c 10 http://fingerprint-api-canary:8000/"
```

### Task 2: Distributed Tracing (Jaeger)

#### 2.1 Jaeger Deployment

```yaml
# monitoring/jaeger/jaeger-deployment.yaml
---
apiVersion: v1
kind: Namespace
metadata:
  name: tracing

---
# Jaeger All-in-One (for dev/staging, use distributed for production)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: jaeger
  namespace: tracing
  labels:
    app: jaeger
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: jaeger
  template:
    metadata:
      labels:
        app: jaeger
        version: v1
    spec:
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - jaeger
              topologyKey: kubernetes.io/hostname
      containers:
      - name: jaeger
        image: jaegertracing/all-in-one:latest
        env:
        - name: COLLECTOR_ZIPKIN_HOST_PORT
          value: ":9411"
        - name: MEMORY_MAX_TRACES
          value: "10000"
        - name: BADGER_EPHEMERAL
          value: "false"
        - name: BADGER_CONSISTENCY
          value: "true"
        ports:
        - containerPort: 5775
          protocol: UDP
          name: zipkin-compact
        - containerPort: 6831
          protocol: UDP
          name: jaeger-compact
        - containerPort: 6832
          protocol: UDP
          name: jaeger-binary
        - containerPort: 5778
          protocol: TCP
          name: serve-configs
        - containerPort: 16686
          protocol: TCP
          name: jaeger-ui
        - containerPort: 14268
          protocol: TCP
          name: jaeger-collector
        - containerPort: 9411
          protocol: TCP
          name: zipkin-collector
        livenessProbe:
          httpGet:
            path: /
            port: 16686
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /
            port: 16686
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 1000m
            memory: 1Gi

---
# Jaeger Service
apiVersion: v1
kind: Service
metadata:
  name: jaeger-collector
  namespace: tracing
  labels:
    app: jaeger
spec:
  selector:
    app: jaeger
  ports:
  - name: jaeger-collector-zipkin-thrift
    port: 14268
    protocol: TCP
    targetPort: 14268
  - name: jaeger-collector-grpc
    port: 14250
    protocol: TCP
    targetPort: 14250
  type: ClusterIP

---
# Jaeger UI Service
apiVersion: v1
kind: Service
metadata:
  name: jaeger-ui
  namespace: tracing
  labels:
    app: jaeger
spec:
  selector:
    app: jaeger
  ports:
  - name: jaeger-ui
    port: 16686
    protocol: TCP
    targetPort: 16686
  type: ClusterIP

---
# Jaeger Ingress
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: jaeger-ui
  namespace: tracing
spec:
  rules:
  - host: jaeger.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: jaeger-ui
            port:
              number: 16686
```

#### 2.2 Istio Tracing Integration

```yaml
# k8s/networking/istio/tracing-config.yaml
---
# Telemetry for distributed tracing
apiVersion: telemetry.istio.io/v1alpha1
kind: Telemetry
metadata:
  name: fingerprint-api-tracing
  namespace: fingerprint-api
spec:
  tracing:
  - providers:
    - name: jaeger
    randomSamplingPercentage: 100.0
    useRequestIdForTraceSampling: true

---
# RequestAuthentication for trace context propagation
apiVersion: security.istio.io/v1beta1
kind: RequestAuthentication
metadata:
  name: jwt-auth
  namespace: fingerprint-api
spec:
  jwtRules:
  - issuer: https://auth.example.com
    jwksUri: https://auth.example.com/.well-known/jwks.json

---
# AuthorizationPolicy for trace context
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: fingerprint-api-authz
  namespace: fingerprint-api
spec:
  selector:
    matchLabels:
      app: fingerprint-api
  action: ALLOW
  rules:
  - from:
    - source:
        principals:
        - cluster.local/ns/fingerprint-api/sa/fingerprint-api
    to:
    - operation:
        methods:
        - GET
        - POST
```

### Task 3: Rate Limiting

#### 3.1 Local Rate Limiting

```yaml
# k8s/networking/istio/rate-limiting.yaml
---
# EnvoyFilter for local rate limiting
apiVersion: networking.istio.io/v1alpha3
kind: EnvoyFilter
metadata:
  name: rate-limiting
  namespace: fingerprint-api
spec:
  workloadSelector:
    labels:
      app: fingerprint-api
  configPatches:
  - applyTo: HTTP_FILTER
    match:
      context: SIDECAR_INBOUND
      listener:
        filterChain:
          filter:
            name: "envoy.filters.network.http_connection_manager"
      filter:
        name: "envoy.filters.http.router"
    patch:
      operation: INSERT_BEFORE
      value:
        name: envoy.filters.http.local_ratelimit
        typedConfig:
          "@type": type.googleapis.com/udpa.type.v1.TypedStruct
          type_url: type.googleapis.com/envoy.extensions.filters.http.local_ratelimit.v3.LocalRateLimit
          value:
            stat_prefix: http_local_rate_limiter
            token_bucket:
              max_tokens: 1000
              tokens_per_fill: 1000
              fill_interval: 1s
            filter_enabled:
              runtime_key: local_rate_limit_enabled
              default_value:
                numerator: 100
                denominator: HUNDRED
            filter_enforced:
              runtime_key: local_rate_limit_enforced
              default_value:
                numerator: 100
                denominator: HUNDRED
            response_headers_to_add:
            - append_action: OVERWRITE_IF_EXISTS_OR_ADD
              header:
                key: x-local-rate-limit
                value: "true"

---
# RequestRate for per-endpoint limits
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: fingerprint-api-ratelimit
  namespace: fingerprint-api
spec:
  hosts:
  - fingerprint-api
  http:
  # Identify endpoint with higher limits
  - name: status-endpoint
    match:
    - uri:
        prefix: /status
    route:
    - destination:
        host: fingerprint-api
        port:
          number: 8000
  # Identify endpoint with default limits
  - name: identify-endpoint
    match:
    - uri:
        prefix: /identify
    route:
    - destination:
        host: fingerprint-api
        port:
          number: 8000
```

### Task 4: Kiali Observability

#### 4.1 Kiali Installation

```yaml
# monitoring/kiali/kiali-deployment.yaml
---
apiVersion: v1
kind: Namespace
metadata:
  name: kiali

---
# Kiali ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: kiali
  namespace: kiali
  labels:
    version: v1.57
    app: kiali
data:
  kiali.yaml: |
    auth:
      strategy: anonymous
    deployment:
      accessible_namespaces:
      - '**'
      instance_name: kiali
    external_services:
      prometheus:
        url: http://prometheus.monitoring:9090
      tracing:
        enabled: true
        namespace_selector: true
        url: http://jaeger-collector.tracing:16686
      grafana:
        enabled: true
        url: http://grafana.monitoring:3000
    identity:
      cert_file: /kiali-secrets/tls.crt
      private_key_file: /kiali-secrets/tls.key
    service:
      node_port: 30030

---
# Kiali Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kiali
  namespace: kiali
  labels:
    app: kiali
    version: v1.57
spec:
  replicas: 2
  selector:
    matchLabels:
      app: kiali
  template:
    metadata:
      labels:
        app: kiali
        version: v1.57
    spec:
      serviceAccountName: kiali
      containers:
      - name: kiali
        image: quay.io/kiali/kiali:latest
        ports:
        - containerPort: 20001
          name: api-port
        - containerPort: 7777
          name: metrics-port
        env:
        - name: ACTIVE_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: LOG_LEVEL
          value: info
        volumeMounts:
        - name: kiali-configuration
          mountPath: /etc/kiali
          readOnly: true
        livenessProbe:
          httpGet:
            path: /kiali/healthz
            port: 20001
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /kiali/healthz
            port: 20001
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          requests:
            cpu: 200m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 512Mi
      volumes:
      - name: kiali-configuration
        configMap:
          name: kiali

---
# Kiali Service
apiVersion: v1
kind: Service
metadata:
  name: kiali
  namespace: kiali
  labels:
    app: kiali
spec:
  selector:
    app: kiali
  ports:
  - port: 20001
    protocol: TCP
    targetPort: 20001
    nodePort: 30030
  type: NodePort

---
# ServiceAccount
apiVersion: v1
kind: ServiceAccount
metadata:
  name: kiali
  namespace: kiali

---
# ClusterRole for Kiali
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: kiali
rules:
- apiGroups: [""]
  resources: ["namespaces", "pods", "services", "endpoints"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets", "statefulsets", "daemonsets"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["networking.istio.io"]
  resources: ["virtualservices", "destinationrules", "gateways", "serviceentries"]
  verbs: ["get", "list", "watch"]

---
# ClusterRoleBinding
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: kiali
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: kiali
subjects:
- kind: ServiceAccount
  name: kiali
  namespace: kiali

---
# Ingress for Kiali UI
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: kiali
  namespace: kiali
spec:
  rules:
  - host: kiali.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: kiali
            port:
              number: 20001
```

### Task 5: Advanced Monitoring Dashboards

#### 5.1 Prometheus ServiceMonitor

```yaml
# monitoring/prometheus/service-monitor.yaml
---
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: fingerprint-api-mesh
  namespace: monitoring
spec:
  selector:
    matchLabels:
      app: fingerprint-api
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics

---
# PrometheusRule for alerting
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: fingerprint-api-mesh-alerts
  namespace: monitoring
spec:
  groups:
  - name: service-mesh.rules
    interval: 30s
    rules:
    # Canary deployment alerts
    - alert: CanaryErrorRateHigh
      expr: |
        (sum(rate(requests_total{response_code=~"5.."}[5m])) by (version)) /
        (sum(rate(requests_total[5m])) by (version)) > 0.05
      for: 2m
      annotations:
        summary: "High error rate in {{ $labels.version }}"
        description: "Error rate is {{ $value | humanizePercentage }}"

    - alert: CanaryLatencyHigh
      expr: |
        histogram_quantile(0.95, rate(request_duration_seconds_bucket[5m])) > 1
      for: 2m
      annotations:
        summary: "High latency in canary deployment"

    # Rate limiting alerts
    - alert: RateLimitExceeded
      expr: increase(envoy_http_ratelimit_ratelimit_status{status="over_limit"}[1m]) > 100
      annotations:
        summary: "Rate limit exceeded on {{ $labels.service }}"

    # Circuit breaker alerts
    - alert: CircuitBreakerTriggered
      expr: increase(envoy_cluster_circuit_breakers_default_cx_open[5m]) > 0
      annotations:
        summary: "Circuit breaker triggered for {{ $labels.cluster_name }}"
```

#### 5.2 Grafana Dashboard for Service Mesh

```yaml
# monitoring/grafana/mesh-dashboard.yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: mesh-dashboard
  namespace: monitoring
data:
  mesh-overview.json: |
    {
      "dashboard": {
        "title": "Service Mesh Overview",
        "panels": [
          {
            "title": "Request Rate (req/sec)",
            "targets": [
              {
                "expr": "sum(rate(requests_total[1m]))"
              }
            ]
          },
          {
            "title": "Error Rate (%)",
            "targets": [
              {
                "expr": "sum(rate(requests_total{response_code=~\"5..\"}[1m])) / sum(rate(requests_total[1m])) * 100"
              }
            ]
          },
          {
            "title": "P99 Latency (ms)",
            "targets": [
              {
                "expr": "histogram_quantile(0.99, rate(request_duration_seconds_bucket[5m])) * 1000"
              }
            ]
          },
          {
            "title": "Canary Traffic Split (%)",
            "targets": [
              {
                "expr": "100 - sum(rate(requests_total{version=\"stable\"}[1m])) / sum(rate(requests_total[1m])) * 100"
              }
            ]
          },
          {
            "title": "Circuit Breaker Status",
            "targets": [
              {
                "expr": "envoy_cluster_circuit_breakers_default_cx_open"
              }
            ]
          }
        ]
      }
    }
```

---

## 🚀 Deployment Sequence

### Step 1: Deploy Jaeger

```bash
kubectl apply -f monitoring/jaeger/jaeger-deployment.yaml

# Verify Jaeger deployment
kubectl get pods -n tracing
kubectl get svc -n tracing
```

### Step 2: Deploy Kiali

```bash
kubectl apply -f monitoring/kiali/kiali-deployment.yaml

# Verify Kiali deployment
kubectl get pods -n kiali
kubectl get svc -n kiali
```

### Step 3: Update Istio Configuration

```bash
# For each region
for context in us-east-1 eu-west-1 ap-southeast-1; do
  kubectl config use-context $context
  kubectl apply -f k8s/networking/istio/tracing-config.yaml
  kubectl apply -f k8s/networking/istio/canary-deployment.yaml
  kubectl apply -f k8s/networking/istio/rate-limiting.yaml
done
```

### Step 4: Update Prometheus

```bash
kubectl apply -f monitoring/prometheus/service-monitor.yaml
kubectl apply -f monitoring/grafana/mesh-dashboard.yaml
```

---

## ✅ Verification Steps

### Verify Canary Deployment

```bash
# Check canary status
kubectl get vs -n fingerprint-api

# View traffic split
kubectl describe vs fingerprint-api-canary -n fingerprint-api

# Test canary header
curl -H "x-user-id: canary-user-1" http://api.example.com/identify
```

### Verify Tracing

```bash
# Check Jaeger pods
kubectl get pods -n tracing

# Port-forward to Jaeger UI
kubectl port-forward svc/jaeger-ui 16686:16686 -n tracing

# Access: http://localhost:16686
# Search for traces from fingerprint-api
```

### Verify Kiali

```bash
# Check Kiali pods
kubectl get pods -n kiali

# Port-forward to Kiali UI
kubectl port-forward svc/kiali 20001:20001 -n kiali

# Access: http://localhost:20001
# Should show service mesh graph with canary deployment
```

### Verify Rate Limiting

```bash
# Generate load to trigger rate limiting
ab -n 5000 -c 100 http://api.example.com/status

# Check rate limit headers
curl -v http://api.example.com/status | grep rate-limit
```

---

## 📊 Success Criteria

| Metric | Target | Status |
|--------|--------|--------|
| **Canary traffic split** | 95/5 stable/canary | ⏳ |
| **Trace sampling** | 100% of requests | ⏳ |
| **Rate limit enforcement** | 1000 req/sec per pod | ⏳ |
| **Kiali mesh visualization** | <2s load time | ⏳ |
| **Circuit breaker accuracy** | <1min detection time | ⏳ |

---

## 🔗 Integration Points

- Jaeger connects to all regional clusters
- Kiali aggregates mesh metrics from Prometheus federation
- Rate limiting enforced at envoy sidecar level
- AuthorizationPolicy integrates with RBAC

---

**Status**: Ready for implementation  
**Estimated Completion**: 5-7 hours  
**Next Phase**: 9.3 Advanced Caching Strategies
