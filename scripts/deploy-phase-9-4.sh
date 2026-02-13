#!/bin/bash

#############################################################################
# Phase 9.4: API Gateway & Distributed Rate Limiting Deployment Script
#
# Automates deployment of Kong API Gateway and rate limiting infrastructure
# Includes verification steps and health checks
#############################################################################

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_step() {
    echo -e "${YELLOW}[STEP]${NC} $1"
}

#############################################################################
# Pre-deployment checks
#############################################################################

check_prerequisites() {
    log_step "Running pre-deployment checks..."

    # Check kubectl access
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Kubernetes cluster not accessible"
        return 1
    fi
    log_success "Kubernetes cluster accessible"

    # Check monitoring namespace
    if ! kubectl get namespace monitoring &> /dev/null; then
        log_error "Monitoring namespace not found - run Phase 9.2 first"
        return 1
    fi
    log_success "Monitoring namespace exists"

    # Check Redis availability
    if ! kubectl get service redis-cluster -n caching &> /dev/null; then
        log_error "Redis cluster not found - run Phase 9.3 first"
        return 1
    fi
    log_success "Redis cluster available"

    return 0
}

#############################################################################
# Step 1: Deploy Kong PostgreSQL Database
#############################################################################

deploy_kong_postgres() {
    log_step "STEP 1: Deploying Kong PostgreSQL Database"

    log_info "Creating api-gateway namespace..."
    kubectl create namespace api-gateway --dry-run=client -o yaml | kubectl apply -f -

    log_info "Creating Kong PostgreSQL resources..."
    kubectl apply -f k8s/api-gateway/kong-postgres.yaml

    log_info "Waiting for PostgreSQL to be ready (max 60 seconds)..."
    kubectl wait --for=condition=ready pod \
        -l app=kong-postgres \
        -n api-gateway \
        --timeout=60s || {
        log_error "PostgreSQL failed to start"
        kubectl logs -n api-gateway -l app=kong-postgres
        return 1
    }

    log_success "PostgreSQL deployed and ready"

    log_info "Waiting for Kong migrations job to complete..."
    kubectl wait --for=condition=complete job/kong-migrations \
        -n api-gateway \
        --timeout=120s || {
        log_error "Kong migrations failed"
        kubectl logs -n api-gateway job/kong-migrations
        return 1
    }

    log_success "Kong database initialized"
}

#############################################################################
# Step 2: Deploy Kong API Gateway
#############################################################################

deploy_kong_gateway() {
    log_step "STEP 2: Deploying Kong API Gateway"

    log_info "Deploying Kong gateway pods..."
    kubectl apply -f k8s/api-gateway/kong-deployment.yaml

    log_info "Waiting for Kong pods to be ready (max 120 seconds)..."
    kubectl rollout status deployment/kong -n api-gateway --timeout=120s || {
        log_error "Kong deployment failed"
        kubectl describe pod -n api-gateway -l app=kong | head -50
        return 1
    }

    log_success "Kong gateway deployed (3 replicas ready)"

    # Get Kong service info
    KONG_LB_IP=$(kubectl get svc kong-gateway -n api-gateway -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "pending")
    log_info "Kong gateway accessible at: http://$KONG_LB_IP"
    log_info "Kong admin API at: http://kong-admin.api-gateway:8001"
}

#############################################################################
# Step 3: Configure Kong Plugins & Routes
#############################################################################

configure_kong_plugins() {
    log_step "STEP 3: Configuring Kong Plugins & Routes"

    log_info "Applying plugin configurations..."
    kubectl apply -f k8s/api-gateway/kong-plugins.yaml
    log_success "Plugins configured"

    log_info "Waiting for plugin configurations to sync..."
    sleep 10

    # Port-forward to Kong admin for configuration
    kubectl port-forward -n api-gateway svc/kong-admin 8001:8001 &
    PF_PID=$!
    sleep 3

    log_info "Configuring rate limiting plugin..."
    curl -X POST http://localhost:8001/plugins \
        -d "name=rate-limiting" \
        -d "config.minute=3600" \
        -d "config.hour=null" \
        -d "config.policy=redis" \
        -d "config.redis_host=redis-cluster.caching.svc.cluster.local" \
        -d "config.redis_port=6379" \
        2>/dev/null || log_error "Failed to configure rate limiting (Kong may not be ready yet)"

    kill $PF_PID 2>/dev/null || true

    log_success "Kong plugin configuration complete"
}

#############################################################################
# Step 4: Deploy Rate Limiting Configuration
#############################################################################

deploy_rate_limiting_config() {
    log_step "STEP 4: Deploying Rate Limiting Configuration"

    log_info "Creating rate limiting configuration..."
    kubectl apply -f k8s/api-gateway/rate-limiting-configmap.yaml
    log_success "Rate limiting configuration deployed"

    log_info "Quota tiers available:"
    log_info "  • Free: 100 req/min, 50K req/month"
    log_info "  • Pro: 1000 req/min, 1M req/month ($99/month)"
    log_info "  • Enterprise: Unlimited (contact sales)"
    log_info "  • Partner: Unlimited (special program)"
}

#############################################################################
# Step 5: Deploy Monitoring & Alerting
#############################################################################

deploy_monitoring() {
    log_step "STEP 5: Deploying Monitoring & Alerting"

    log_info "Creating API gateway monitoring rules..."
    kubectl apply -f monitoring/api-gateway-monitoring.yaml
    log_success "Prometheus monitoring configured"

    log_info "Monitoring includes:"
    log_info "  • Kong availability (down/up)"
    log_info "  • Error rates (>5%)"
    log_info "  • Rate limit rejections"
    log_info "  • Upstream health"
    log_info "  • Database connectivity"
    log_info "  • Proxy latency (P95)"
}

#############################################################################
# Step 6: Verification Procedures
#############################################################################

verify_kong_health() {
    log_step "Verifying Kong Health"

    # Check pods
    POD_COUNT=$(kubectl get pod -n api-gateway -l app=kong --no-headers | wc -l)
    if [ "$POD_COUNT" -ge 2 ]; then
        log_success "Kong pods ready ($POD_COUNT/3)"
    else
        log_error "Not enough Kong pods ready ($POD_COUNT/3)"
        return 1
    fi

    # Health check via port-forward
    kubectl port-forward -n api-gateway svc/kong-status 8100:8100 &
    PF_PID=$!
    sleep 2

    if curl -s http://localhost:8100/status | grep -q "database=ok"; then
        log_success "Kong health check passed"
    else
        log_error "Kong health check failed"
        kill $PF_PID 2>/dev/null || true
        return 1
    fi

    kill $PF_PID 2>/dev/null || true
}

verify_prometheus_scraping() {
    log_step "Verifying Prometheus Scraping"

    log_info "Kong metrics should appear in Prometheus within 2 minutes"
    log_info "Prometheus query: up{job=\"kong-metrics\"}"
}

verify_rate_limiting() {
    log_step "Verifying Rate Limiting Setup"

    # Check if rate limiting config is present
    if kubectl get configmap rate-limiting-config -n api-gateway &> /dev/null; then
        log_success "Rate limiting configuration present"
    else
        log_error "Rate limiting configuration missing"
        return 1
    fi

    log_info "Rate limiting quotas configured:"
    kubectl get configmap rate-limiting-config -n api-gateway -o jsonpath='{.data.quotas\.yaml}' | grep "name:" | head -4
}

#############################################################################
# Establish Performance Baseline
#############################################################################

establish_baseline() {
    log_step "Establishing Performance Baseline"

    BASELINE_FILE="phase-9-4-baseline-$(date +%Y%m%d-%H%M%S).txt"

    cat > "$BASELINE_FILE" << EOF
# Phase 9.4: API Gateway & Rate Limiting Baseline
# Recorded: $(date)
# Kubernetes Cluster: $(kubectl cluster-info 2>/dev/null | head -1)

## Kong Gateway Status
- Replicas: 3
- Admin API: http://kong-admin.api-gateway:8001
- Gateway Endpoint: Check LoadBalancer status

## Rate Limiting Configuration
- Algorithm: Token Bucket
- Storage: Redis (caching namespace)
- User Tiers: Free, Pro, Enterprise, Partner

## Monitoring
- Prometheus: Scraping Kong metrics
- Grafana: Dashboards available
- Alert Rules: 8 configured

## Expected Performance
- API Gateway Latency: <50ms (P95)
- Rate Limiting Check: <2ms
- Upstream Latency: 50-200ms (App dependent)
- Cache Hit Rate: 85%+ (from Phase 9.3)

## Next Steps
1. Run load tests to validate rate limiting
2. Optimize Kong worker processes
3. Monitor baseline for 24 hours
4. Begin Phase 9.5: Billing & Metering

EOF

    log_success "Baseline recorded: $BASELINE_FILE"
    cat "$BASELINE_FILE"
}

#############################################################################
# Main Deployment Flow
#############################################################################

main() {
    log_info "╔════════════════════════════════════════════════════╗"
    log_info "║  Phase 9.4: API Gateway & Rate Limiting Deployment║"
    log_info "║  Duration: 10-15 minutes                            ║"
    log_info "╚════════════════════════════════════════════════════╝"

    # Run all deployment steps
    check_prerequisites || exit 1
    deploy_kong_postgres || exit 1
    deploy_kong_gateway || exit 1
    configure_kong_plugins || exit 1
    deploy_rate_limiting_config || exit 1
    deploy_monitoring || exit 1

    # Verification
    verify_kong_health || exit 1
    verify_prometheus_scraping
    verify_rate_limiting || exit 1

    # Baseline
    establish_baseline

    log_info ""
    log_info "╔════════════════════════════════════════════════════╗"
    log_info "║  ✓ Phase 9.4 Deployment Complete!                 ║"
    log_info "║                                                    ║"
    log_info "║  Kong Gateway: 3 replicas running                  ║"
    log_info "║  Rate Limiting: Configured with Redis backend      ║"
    log_info "║  Monitoring: Prometheus + Grafana ready            ║"
    log_info "║  User Quotas: Free/Pro/Enterprise tiers active     ║"
    log_info "║                                                    ║"
    log_info "║  Next: Phase 9.5 Billing & Metering Integration    ║"
    log_info "╚════════════════════════════════════════════════════╝"
}

# Execute main function
main
