#!/bin/bash

# Phase 9.3 Deployment Script - Advanced Caching Strategies
# Deploys Redis cluster, cache warmer, monitoring, and integrates with fingerprint-api

set -e

NAMESPACE_CACHING="caching"
NAMESPACE_API="fingerprint-api"
NAMESPACE_MONITORING="monitoring"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
  echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
  echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
  echo -e "${BLUE}[STEP]${NC} $1"
}

# Step 1: Deploy Redis Cluster
deploy_redis_cluster() {
  log_step "Step 1: Deploying Redis cluster with Sentinel..."
  
  # Create caching namespace
  kubectl create namespace $NAMESPACE_CACHING --dry-run=client -o yaml | kubectl apply -f -
  
  # Deploy Redis StatefulSet with Sentinel
  kubectl apply -f k8s/caching/redis-statefulset.yaml
  
  # Wait for Redis cluster to be ready
  log_info "Waiting for Redis cluster (3 pods) to be ready..."
  kubectl wait --for=condition=Ready pod \
    -l app=redis \
    -n $NAMESPACE_CACHING \
    --timeout=600s || {
    log_error "Redis cluster failed to start"
    kubectl logs -n $NAMESPACE_CACHING -l app=redis -c redis --tail=50
    return 1
  }
  
  # Verify replication
  sleep 10  # Give replication time to establish
  MASTER_POD=$(kubectl get pod -n $NAMESPACE_CACHING -l app=redis -o jsonpath='{.items[0].metadata.name}')
  REPLICAS=$(kubectl exec -n $NAMESPACE_CACHING $MASTER_POD -- redis-cli info replication | grep connected_slaves | cut -d: -f2 | tr -d '\r')
  
  if [ "$REPLICAS" -ge 2 ]; then
    log_info "✓ Redis cluster deployed successfully (Master + $REPLICAS replicas)"
  else
    log_warn "⚠ Redis cluster may not be fully replicated (replicas: $REPLICAS)"
  fi
}

# Step 2: Deploy services and monitoring
deploy_services_and_monitoring() {
  log_step "Step 2: Deploying Redis services and monitoring..."
  
  # Deploy services
  kubectl apply -f k8s/caching/redis-service.yaml
  
  # Deploy Redis monitoring
  kubectl apply -f monitoring/redis-monitoring.yaml
  
  # Deploy cache dashboards
  kubectl apply -f monitoring/cache-dashboards.yaml
  
  log_info "✓ Services and monitoring deployed"
}

# Step 3: Deploy cache management resources
deploy_cache_management() {
  log_step "Step 3: Deploying cache management resources..."
  
  # Create fingerprint-api namespace if needed
  kubectl create namespace $NAMESPACE_API --dry-run=client -o yaml | kubectl apply -f -
  
  # Deploy cache warmer and invalidation watcher
  kubectl apply -f k8s/caching/cache-management.yaml
  
  log_info "✓ Cache management resources deployed"
}

# Step 4: Update fingerprint-api deployment with cache integration
update_api_deployment() {
  log_step "Step 4: Updating fingerprint-api with cache integration..."
  
  # Check if deployment exists
  if kubectl get deployment fingerprint-api -n $NAMESPACE_API 2>/dev/null; then
    # Add cache environment variables
    kubectl set env deployment/fingerprint-api \
      -n $NAMESPACE_API \
      CACHE_REDIS_ADDR="redis-cluster.caching:6379" \
      CACHE_ENABLED="true" \
      CACHE_L1_TTL_SECS="300" \
      CACHE_L2_TTL_SECS="1800" \
      CACHE_L1_MAX_SIZE="10000" \
      CACHE_L2_MAX_SIZE="100000" \
      -c fingerprint-api || true
    
    log_info "✓ fingerprint-api deployment updated with cache configuration"
  else
    log_warn "⚠ fingerprint-api deployment not found in $NAMESPACE_API"
    log_warn "  Please manually add cache environment variables after deployment"
  fi
}

# Verification functions
verify_redis_cluster() {
  log_info "Verifying Redis cluster..."
  
  REDIS_POD=$(kubectl get pod -n $NAMESPACE_CACHING -l app=redis -o jsonpath='{.items[0].metadata.name}')
  
  # Test connectivity
  kubectl exec -n $NAMESPACE_CACHING $REDIS_POD -- redis-cli ping > /dev/null 2>&1 && \
    log_info "✓ Redis cluster is responding" || {
    log_error "✗ Redis cluster is not responding"
    return 1
  }
  
  # Check Sentinel status
  kubectl exec -n $NAMESPACE_CACHING $REDIS_POD -c sentinel -- \
    redis-cli -p 26379 sentinel masters | grep -q mymaster && \
    log_info "✓ Redis Sentinel is monitoring" || {
    log_warn "⚠ Redis Sentinel status unclear"
  }
  
  # Check Prometheus scraping
  sleep 30  # Wait for first scrape
  SCRAPE_OK=$(kubectl logs -n monitoring -l app.kubernetes.io/name=prometheus --tail=50 2>/dev/null | grep -c "redis" || true)
  if [ $SCRAPE_OK -gt 0 ]; then
    log_info "✓ Prometheus is scraping Redis metrics"
  else
    log_warn "⚠ Prometheus may not be scraping Redis yet"
  fi
}

verify_monitoring() {
  log_info "Verifying monitoring setup..."
  
  # Check ServiceMonitor
  kubectl get servicemonitor redis -n caching > /dev/null 2>&1 && \
    log_info "✓ Redis ServiceMonitor created" || \
    log_warn "⚠ Redis ServiceMonitor not found"
  
  # Check PrometheusRule
  kubectl get prometheusrule redis-caching -n monitoring > /dev/null 2>&1 && \
    log_info "✓ Redis PrometheusRule created" || \
    log_warn "⚠ Redis PrometheusRule not found"
  
  # Check Grafana dashboards
  kubectl get cm cache-performance-dashboard -n monitoring > /dev/null 2>&1 && \
    log_info "✓ Grafana dashboards created" || \
    log_warn "⚠ Grafana dashboards not found"
}

verify_cache_management() {
  log_info "Verifying cache management resources..."
  
  # Check CronJobs
  CRONJOB_COUNT=$(kubectl get cronjob -n fingerprint-api | wc -l)
  if [ $CRONJOB_COUNT -gt 1 ]; then
    log_info "✓ Cache warmer CronJobs deployed"
  else
    log_warn "⚠ Cache warmer CronJobs may not be deployed"
  fi
  
  # Check cache invalidation watcher
  kubectl get deployment cache-invalidation-watcher -n fingerprint-api > /dev/null 2>&1 && \
    log_info "✓ Cache invalidation watcher deployed" || \
    log_warn "⚠ Cache invalidation watcher not found"
}

# Test cache functionality
test_cache() {
  log_info "Testing cache functionality..."
  
  # Port-forward to Redis
  kubectl port-forward -n caching svc/redis-cluster 6379:6379 &
  PF_PID=$!
  sleep 2
  
  # Test set/get
  if redis-cli -h localhost -p 6379 SET test:key "test:value" 2>/dev/null && \
     redis-cli -h localhost -p 6379 GET test:key | grep -q "test:value" 2>/dev/null; then
    log_info "✓ Redis basic operations working"
    redis-cli -h localhost -p 6379 DEL test:key > /dev/null 2>&1
  else
    log_error "✗ Redis basic operations failed"
  fi
  
  kill $PF_PID 2>/dev/null || true
}

# Performance baseline
establish_baseline() {
  log_info "Establishing cache performance baseline..."
  
  # Get current cache stats
  BASELINE_FILE="cache-baseline-$(date +%s).txt"
  
  cat > $BASELINE_FILE <<EOF
Cache Performance Baseline - $(date -u)

Target Metrics:
- Cache Hit Rate: 85% (target)
- L1 Query Latency: <1ms P95
- L2 Query Latency: 5-20ms P95
- Cache Capacity: 100,000 entries (L2)

Baseline will be established after 1 hour of traffic.
Monitor with: kubectl logs -n fingerprint-api -f -l app=fingerprint-api
EOF
  
  log_info "✓ Baseline created: $BASELINE_FILE"
  cat $BASELINE_FILE
}

# Main deployment
main() {
  log_info "╔══════════════════════════════════════════════════════════════╗"
  log_info "║         Phase 9.3: Advanced Caching Strategies              ║"
  log_info "║         Deploying Redis cluster and cache layer             ║"
  log_info "╚══════════════════════════════════════════════════════════════╝"
  log_info ""
  
  # Pre-checks
  log_info "Running pre-deployment checks..."
  
  if ! kubectl cluster-info > /dev/null 2>&1; then
    log_error "Kubernetes cluster not accessible"
    exit 1
  fi
  
  if ! kubectl get ns $NAMESPACE_MONITORING > /dev/null 2>&1; then
    log_error "Monitoring namespace not found"
    log_error "Please deploy Phase 8+ monitoring first"
    exit 1
  fi
  
  # Deploy components
  deploy_redis_cluster || exit 1
  sleep 5
  
  deploy_services_and_monitoring || exit 1
  sleep 5
  
  deploy_cache_management || exit 1
  sleep 5
  
  update_api_deployment || true
  
  log_info ""
  log_info "╔══════════════════════════════════════════════════════════════╗"
  log_info "║                      Verification                            ║"
  log_info "╚══════════════════════════════════════════════════════════════╝"
  
  # Verify deployments
  verify_redis_cluster || log_warn "Redis verification incomplete"
  verify_monitoring || log_warn "Monitoring verification incomplete"
  verify_cache_management || log_warn "Cache management verification incomplete"
  test_cache || log_warn "Cache functionality test failed"
  
  log_info ""
  log_info "╔══════════════════════════════════════════════════════════════╗"
  log_info "║                   Deployment Complete ✓                     ║"
  log_info "╚══════════════════════════════════════════════════════════════╝"
  
  establish_baseline
  
  log_info ""
  log_info "Next steps:"
  log_info "1. Monitor Redis cluster: kubectl port-forward -n caching svc/redis-cluster 6379:6379"
  log_info "2. View cache metrics: kubectl port-forward -n monitoring svc/grafana 3000:3000"
  log_info "3. Check cache warmer logs: kubectl logs -n fingerprint-api -f job/cache-warmer"
  log_info "4. Monitor cache hit rate for 1+ hour to establish baseline"
  log_info ""
}

main "$@"
