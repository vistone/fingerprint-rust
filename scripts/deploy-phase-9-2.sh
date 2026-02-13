#!/bin/bash

# Phase 9.2 Deployment Script - Service Mesh Advanced Features
# This script deploys Jaeger, Kiali, canary deployments, and advanced monitoring

set -e

NAMESPACE_API="fingerprint-api"
NAMESPACE_MONITORING="monitoring"
NAMESPACE_KIALI="kiali"
NAMESPACE_TRACING="tracing"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
  echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

# Step 1: Deploy Jaeger (Distributed Tracing)
deploy_jaeger() {
  log_info "Step 1: Deploying Jaeger distributed tracing..."
  
  kubectl apply -f monitoring/jaeger/jaeger-deployment.yaml
  
  # Wait for Jaeger deployment
  kubectl wait --for=condition=available --timeout=300s \
    deployment/jaeger -n $NAMESPACE_TRACING || {
    log_error "Jaeger deployment failed"
    return 1
  }
  
  log_info "✓ Jaeger deployed successfully"
  log_info "  Jaeger UI: kubectl port-forward -n tracing svc/jaeger-ui 16686:16686"
}

# Step 2: Deploy Kiali (Service Mesh Visualization)
deploy_kiali() {
  log_info "Step 2: Deploying Kiali service mesh observability..."
  
  kubectl apply -f monitoring/kiali/kiali-deployment.yaml
  
  # Wait for Kiali deployment
  kubectl wait --for=condition=available --timeout=300s \
    deployment/kiali -n $NAMESPACE_KIALI || {
    log_error "Kiali deployment failed"
    return 1
  }
  
  log_info "✓ Kiali deployed successfully"
  log_info "  Kiali UI: kubectl port-forward -n kiali svc/kiali 20001:20001"
}

# Step 3: Deploy Istio Telemetry & Security Policies
deploy_istio_telemetry() {
  log_info "Step 3: Deploying Istio telemetry and security policies..."
  
  # Create namespaces if they don't exist
  kubectl create namespace $NAMESPACE_API --dry-run=client -o yaml | kubectl apply -f -
  
  # Deploy telemetry configuration
  kubectl apply -f k8s/networking/istio/telemetry-config.yaml
  
  log_info "✓ Istio telemetry policies deployed"
}

# Step 4: Deploy Canary Deployment Infrastructure
deploy_canary() {
  log_info "Step 4: Deploying canary deployment infrastructure..."
  
  # Deploy VirtualService and rate limiting
  kubectl apply -f k8s/networking/canary/virtualservice.yaml
  kubectl apply -f k8s/networking/canary/rate-limiting.yaml
  
  # Deploy Flagger Canary CRD (if Flagger is installed)
  if kubectl get crd canaries.flagger.app 2>/dev/null; then
    log_info "  Flagger CRD found, deploying canary manifests..."
    kubectl apply -f k8s/networking/canary/flagger-canary.yaml
  else
    log_warn "Flagger not found, skipping Canary CRD deployment"
    log_info "  To install Flagger, run: helm repo add flagger https://flagger.app && helm install flagger flagger/flagger --namespace istio-system"
  fi
  
  log_info "✓ Canary deployment infrastructure deployed"
}

# Step 5: Deploy Advanced Monitoring (Prometheus Rules & Dashboards)
deploy_monitoring() {
  log_info "Step 5: Deploying advanced monitoring rules and dashboards..."
  
  # Deploy Prometheus rules
  kubectl apply -f monitoring/prometheus-rules-advanced.yaml
  
  # Deploy ServiceMonitors
  kubectl apply -f monitoring/servicemonitor.yaml
  
  # Deploy Grafana dashboards
  kubectl apply -f monitoring/grafana-dashboards-advanced.yaml
  
  log_info "✓ Advanced monitoring deployed"
}

# Verification functions
verify_jaeger() {
  log_info "Verifying Jaeger deployment..."
  
  # Check pod status
  if kubectl get pods -n $NAMESPACE_TRACING | grepjaeger | grep Running; then
    log_info "✓ Jaeger pods running"
    
    # Test collector endpoint
    JAEGER_POD=$(kubectl get pod -n $NAMESPACE_TRACING -l app=jaeger -o jsonpath='{.items[0].metadata.name}')
    kubectl exec -n $NAMESPACE_TRACING $JAEGER_POD -- curl -s http://localhost:16686/ > /dev/null && \
      log_info "✓ Jaeger UI accessible" || log_warn "⚠ Jaeger UI not responding"
    
    return 0
  else
    log_error "✗ Jaeger pods not running"
    return 1
  fi
}

verify_kiali() {
  log_info "Verifying Kiali deployment..."
  
  # Check pod status
  if kubectl get pods -n $NAMESPACE_KIALI | grep kiali | grep Running; then
    log_info "✓ Kiali pods running"
    
    # Test UI endpoint
    KIALI_POD=$(kubectl get pod -n $NAMESPACE_KIALI -l app=kiali -o jsonpath='{.items[0].metadata.name}')
    kubectl exec -n $NAMESPACE_KIALI $KIALI_POD -- curl -s http://localhost:20001/kiali/healthz > /dev/null && \
      log_info "✓ Kiali health check passed" || log_warn "⚠ Kiali health check failed"
    
    return 0
  else
    log_error "✗ Kiali pods not running"
    return 1
  fi
}

verify_telemetry() {
  log_info "Verifying telemetry configuration..."
  
  if kubectl get telemetries.telemetry.istio.io -n $NAMESPACE_API > /dev/null 2>&1; then
    log_info "✓ Telemetry policies found"
    return 0
  else
    log_warn "⚠ No telemetry policies found"
    return 1
  fi
}

verify_canary() {
  log_info "Verifying canary deployment..."
  
  if kubectl get vs fingerprint-api-canary -n $NAMESPACE_API > /dev/null 2>&1; then
    log_info "✓ Canary VirtualService deployed"
  else
    log_warn "⚠ Canary VirtualService not found"
  fi
  
  if kubectl get envoyfilters -n $NAMESPACE_API | grep rate-limiting > /dev/null; then
    log_info "✓ Rate limiting EnvoyFilter deployed"
  else
    log_warn "⚠ Rate limiting EnvoyFilter not found"
  fi
}

verify_monitoring() {
  log_info "Verifying monitoring deployment..."
  
  if kubectl get prometheusrules -n $NAMESPACE_MONITORING | grep service-mesh > /dev/null; then
    log_info "✓ Prometheus rules deployed"
  else
    log_warn "⚠ Prometheus rules not found"
  fi
  
  if kubectl get servicemonitors | grep -E "istio-mesh|jaeger|kiali" > /dev/null; then
    log_info "✓ ServiceMonitors deployed"
  else
    log_warn "⚠ ServiceMonitors not found"
  fi
}

# Main deployment
main() {
  log_info "================== Phase 9.2 Deployment =================="
  log_info "Service Mesh Advanced Features"
  log_info "======================================================"
  
  # Pre-checks
  log_info "Running pre-deployment checks..."
  
  if ! kubectl cluster-info > /dev/null 2>&1; then
    log_error "Kubernetes cluster not accessible"
    exit 1
  fi
  
  if ! kubectl get ns $NAMESPACE_MONITORING > /dev/null 2>&1; then
    log_error "Monitoring namespace not found. Run Phase 8 deployment first."
    exit 1
  fi
  
  # Deploy components
  deploy_jaeger || exit 1
  sleep 5
  
  deploy_kiali || exit 1
  sleep 5
  
  deploy_istio_telemetry || exit 1
  sleep 5
  
  deploy_canary || exit 1
  sleep 5
  
  deploy_monitoring || exit 1
  
  log_info ""
  log_info "================== Verification =================="
  
  # Verify deployments
  verify_jaeger || log_warn "Jaeger verification failed"
  verify_kiali || log_warn "Kiali verification failed"
  verify_telemetry || log_warn "Telemetry verification failed"
  verify_canary || log_warn "Canary verification failed"
  verify_monitoring || log_warn "Monitoring verification failed"
  
  log_info ""
  log_info "================== Deployment Complete =================="
  log_info "Phase 9.2: Service Mesh Advanced Features deployed!"
  log_info ""
  log_info "Next steps:"
  log_info "1. Access Jaeger UI: kubectl port-forward -n tracing svc/jaeger-ui 16686:16686"
  log_info "2. Access Kiali UI: kubectl port-forward -n kiali svc/kiali 20001:20001"
  log_info "3. Configure canary deployments from Flagger documentation"
  log_info "4. Monitor advanced metrics in Grafana"
  log_info ""
}

main "$@"
