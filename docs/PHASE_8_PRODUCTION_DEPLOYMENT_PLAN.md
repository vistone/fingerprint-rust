# Phase 8: 生产部署与监控集成规划

**状态**: Phase 8 INITIALIZATION  
**日期**: 2026-02-13  
**前置完成**: Phase 7.4 REST API (77% 项目进度)  

---

## 📋 Phase 8 总体目标

将 Phase 7.4 完成的生产级 REST API 部署至生产环境，并建立完整的监控、告警和日志系统。

**目标成果**:
- Kubernetes 部署配置  
- Prometheus 监控系统
- ELK 日志聚合
- Grafana 可视化仪表板
- 告警规则和 SLA 管理
- 完整的运维文档

**时间估算**: 12-16 小时（分阶段）

---

## 🎯 Phase 8 工作分解

### 阶段 1: Kubernetes 部署 (4小时)

#### 任务 8.1.1: 创建 Kubernetes 清单文件

**文件结构**:
```
k8s/
├── namespace.yaml              # 命名空间
├── deployment.yaml             # 部署配置
├── service.yaml                # 服务定义
├── configmap.yaml              # 配置映射
├── secret.yaml                 # 密钥管理
├── ingress.yaml                # Ingress 配置
├── hpa.yaml                    # 水平自动扩展
├── pdb.yaml                    # Pod 干扰预算
├── network-policy.yaml         # 网络策略
└── rbac.yaml                   # 角色访问控制
```

**deployment.yaml 要点**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fingerprint-api
  namespace: fingerprint
  labels:
    app: fingerprint-api
    version: 1.0

spec:
  replicas: 3  # 3 个副本
  
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  
  template:
    metadata:
      labels:
        app: fingerprint-api
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8000"
        prometheus.io/path: "/metrics"
    
    spec:
      containers:
      - name: api
        image: fingerprint-api:7.4
        imagePullPolicy: Always
        
        ports:
        - name: http
          containerPort: 8000
          protocol: TCP
        
        env:
        - name: LOG_LEVEL
          value: "info"
        - name: WORKERS
          value: "4"
        
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
        
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        
        readinessProbe:
          httpGet:
            path: /api/v1/models/status
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        
        volumeMounts:
        - name: models
          mountPath: /app/models
          readOnly: true
        - name: config
          mountPath: /app/config
          readOnly: true
      
      volumes:
      - name: models
        configMap:
          name: fingerprint-models
      - name: config
        configMap:
          name: fingerprint-config
      
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
                  - fingerprint-api
              topologyKey: kubernetes.io/hostname
```

**service.yaml**:
```yaml
apiVersion: v1
kind: Service
metadata:
  name: fingerprint-api
  namespace: fingerprint

spec:
  type: ClusterIP
  ports:
  - name: http
    port: 80
    targetPort: 8000
    protocol: TCP
  
  selector:
    app: fingerprint-api
```

**ingress.yaml**:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: fingerprint-api
  namespace: fingerprint
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"

spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.fingerprint.example.com
    secretName: fingerprint-tls
  
  rules:
  - host: api.fingerprint.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: fingerprint-api
            port:
              number: 80
```

**hpa.yaml**:
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: fingerprint-api
  namespace: fingerprint

spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: fingerprint-api
  
  minReplicas: 3
  maxReplicas: 10
  
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

#### 任务 8.1.2: 部署工具和脚本

**deploy.sh** - 自动部署脚本:
```bash
#!/bin/bash
set -e

NAMESPACE="fingerprint"
VERSION="7.4"
ENVIRONMENT="${1:-staging}"

echo "🚀 Deploying fingerprint-api v${VERSION} to ${ENVIRONMENT}..."

# 1. 创建命名空间
kubectl apply -f k8s/namespace.yaml

# 2. 创建配置和密钥
kubectl -n ${NAMESPACE} create configmap fingerprint-config \
  --from-file=config/ --dry-run=client -o yaml | kubectl apply -f -

kubectl -n ${NAMESPACE} create secret generic fingerprint-secrets \
  --from-env-file=.env.${ENVIRONMENT} --dry-run=client -o yaml | kubectl apply -f -

# 3. 部署应用
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/hpa.yaml

# 4. 检查部署状态
echo "⏳ Waiting for deployment to be ready..."
kubectl -n ${NAMESPACE} rollout status deployment/fingerprint-api --timeout=5m

echo "✅ Deployment successful!"
kubectl -n ${NAMESPACE} get pods, svc, ingress
```

### 阶段 2: Prometheus 监控 (3小时)

#### 任务 8.2.1: Prometheus 配置

**prometheus.yml**:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'production'
    environment: 'prod'

alerting:
  alertmanagers:
  - static_configs:
    - targets:
      - alertmanager:9093

rule_files:
  - 'alert_rules.yml'
  - 'recording_rules.yml'

scrape_configs:
  # Kubernetes API
  - job_name: 'kubernetes-apiservers'
    kubernetes_sd_configs:
    - role: endpoints
    scheme: https
    tls_config:
      ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
  
  # Fingerprint API
  - job_name: 'fingerprint-api'
    kubernetes_sd_configs:
    - role: pod
      namespaces:
        names:
        - fingerprint
    relabel_configs:
    - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
      action: keep
      regex: true
    - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
      action: replace
      target_label: __metrics_path__
      regex: (.+)
    - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
      action: replace
      regex: ([^:]+)(?::\d+)?;(\d+)
      replacement: $1:$2
      target_label: __address__
  
  # Node Exporter
  - job_name: 'node'
    static_configs:
    - targets:
      - 'node-exporter:9100'
```

#### 任务 8.2.2: 告警规则

**alert_rules.yml**:
```yaml
groups:
- name: fingerprint_alerts
  interval: 30s
  
  rules:
  # API 可用性告警
  - alert: FingerprintAPIDown
    expr: up{job="fingerprint-api"} == 0
    for: 2m
    annotations:
      summary: "Fingerprint API is down"
      description: "{{ $labels.instance }} has been down for 2 minutes"
  
  # 高错误率
  - alert: HighErrorRate
    expr: |
      rate(fingerprint_api_errors_total[5m]) / rate(fingerprint_api_requests_total[5m]) > 0.05
    for: 5m
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value | humanizePercentage }}"
  
  # 高延迟
  - alert: HighLatency
    expr: |
      histogram_quantile(0.95, fingerprint_api_request_duration_seconds_bucket) > 0.1
    for: 5m
    annotations:
      summary: "High request latency"
      description: "P95 latency is {{ $value }}s"
  
  # 内存不足
  - alert: HighMemoryUsage
    expr: |
      container_memory_usage_bytes{pod_name=~"fingerprint-api.*"} / container_spec_memory_limit_bytes > 0.8
    for: 5m
    annotations:
      summary: "High memory usage"
      description: "Pod {{ $labels.pod_name }} memory usage is {{ $value | humanizePercentage }}"
  
  # CPU 不足
  - alert: HighCPUUsage
    expr: |
      rate(container_cpu_usage_seconds_total{pod_name=~"fingerprint-api.*"}[5m]) > 0.8
    for: 5m
    annotations:
      summary: "High CPU usage"
      description: "Pod {{ $labels.pod_name }} CPU usage is {{ $value | humanizePercentage }}"
  
  # 模型加载失败
  - alert: ModelLoadFailure
    expr: fingerprint_model_load_failure_total > 0
    for: 1m
    annotations:
      summary: "Model loading failure detected"
      description: "{{ $labels.model_name }} failed to load"
```

**recording_rules.yml**:
```yaml
groups:
- name: fingerprint_metrics
  interval: 1m
  
  rules:
  # 请求速率
  - record: fingerprint:request_rate:1m
    expr: rate(fingerprint_api_requests_total[1m])
  
  - record: fingerprint:error_rate:1m
    expr: rate(fingerprint_api_errors_total[1m])
  
  # 平均延迟
  - record: fingerprint:latency:p50
    expr: histogram_quantile(0.50, fingerprint_api_request_duration_seconds_bucket)
  
  - record: fingerprint:latency:p95
    expr: histogram_quantile(0.95, fingerprint_api_request_duration_seconds_bucket)
  
  - record: fingerprint:latency:p99
    expr: histogram_quantile(0.99, fingerprint_api_request_duration_seconds_bucket)
  
  # 可用性
  - record: fingerprint:availability:5m
    expr: |
      (1 - rate(fingerprint_api_errors_total[5m]) / rate(fingerprint_api_requests_total[5m])) * 100
```

### 阶段 3: ELK 日志聚合 (3小时)

#### 任务 8.3.1: Elasticsearch 配置

#### 任务 8.3.2: Logstash 管道

#### 任务 8.3.3: Kibana 仪表板

### 阶段 4: Grafana 可视化 (2小时)

#### 任务 8.4.1: 仪表板配置

### 阶段 5: 运维文档 (3小时)

#### 任务 8.5.1: 部署指南

#### 任务 8.5.2: 运维手册

#### 任务 8.5.3: 故障排查指南

---

## 🚀 建议启动顺序

**第一天 (8小时)**:
- 8.1: Kubernetes 部署 (4h)
- 8.2: Prometheus 监控 (4h)

**第二天 (8小时)**:
- 8.3: ELK 日志系统 (3h)
- 8.4: Grafana 仪表板 (2h)
- 8.5: 运维文档 (3h)

---

## 📊 预期成果

| 组件 | 功能 | 状态 |
|------|------|------|
| K8s | 高可用部署 | 📋 待开发 |
| Prometheus | 指标收集 | 📋 待开发 |
| ELK | 日志聚合 | 📋 待开发 |
| Grafana | 可视化 | 📋 待开发 |
| 文档 | 运维指南 | 📋 待开发 |

---

**下一步**: 立即启动任务 8.1 - Kubernetes 部署配置开发

需要开始吗? (Y/N)
