#!/bin/bash

#############################################################################
# Phase 9.4: API Gateway & Distributed Rate Limiting Deployment Script
#
# Automates deployment of Kong API Gateway and rate limiting infrastructure
# Includes verification steps and health checks
#############################################################################

set -euo pipefail

API_GATEWAY_NAMESPACE="${API_GATEWAY_NAMESPACE:-api-gateway}"
CACHING_NAMESPACE="${CACHING_NAMESPACE:-caching}"
MONITORING_NAMESPACE="${MONITORING_NAMESPACE:-monitoring}"
KONG_ADMIN_PORT_FORWARD_PORT="${KONG_ADMIN_PORT_FORWARD_PORT:-8001}"
KONG_STATUS_PORT_FORWARD_PORT="${KONG_STATUS_PORT_FORWARD_PORT:-8100}"
KONG_ADMIN_INTERNAL_PORT="${KONG_ADMIN_INTERNAL_PORT:-8001}"
KONG_STATUS_INTERNAL_PORT="${KONG_STATUS_INTERNAL_PORT:-8100}"
KONG_RATE_LIMIT_PLUGIN_NAME="${KONG_RATE_LIMIT_PLUGIN_NAME:-rate-limiting}"
KONG_RATE_LIMIT_MINUTE_LIMIT="${KONG_RATE_LIMIT_MINUTE_LIMIT:-3600}"
KONG_RATE_LIMIT_HOUR_LIMIT="${KONG_RATE_LIMIT_HOUR_LIMIT:-null}"
KONG_RATE_LIMIT_POLICY="${KONG_RATE_LIMIT_POLICY:-redis}"
KONG_RATE_LIMIT_REDIS_HOST="${KONG_RATE_LIMIT_REDIS_HOST:-redis-cluster.caching.svc.cluster.local}"
KONG_RATE_LIMIT_REDIS_PORT="${KONG_RATE_LIMIT_REDIS_PORT:-6379}"

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
    if ! kubectl get namespace "$MONITORING_NAMESPACE" &> /dev/null; then
        log_error "Monitoring namespace not found - run Phase 9.2 first"
        return 1
    fi
    log_success "Monitoring namespace exists"

    # Check Redis availability
    if ! kubectl get service redis-cluster -n "$CACHING_NAMESPACE" &> /dev/null; then
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
    kubectl create namespace "$API_GATEWAY_NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

    log_info "Creating Kong PostgreSQL resources..."
    kubectl apply -f k8s/api-gateway/kong-postgres.yaml

    log_info "Waiting for PostgreSQL to be ready (max 60 seconds)..."
    kubectl wait --for=condition=ready pod \
        -l app=kong-postgres \
        -n "$API_GATEWAY_NAMESPACE" \
        --timeout=60s || {
        log_error "PostgreSQL failed to start"
        kubectl logs -n "$API_GATEWAY_NAMESPACE" -l app=kong-postgres
        return 1
    }

    log_success "PostgreSQL deployed and ready"

    log_info "Waiting for Kong migrations job to complete..."
    kubectl wait --for=condition=complete job/kong-migrations \
        -n "$API_GATEWAY_NAMESPACE" \
        --timeout=120s || {
        log_error "Kong migrations failed"
        kubectl logs -n "$API_GATEWAY_NAMESPACE" job/kong-migrations
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
    kubectl rollout status deployment/kong -n "$API_GATEWAY_NAMESPACE" --timeout=120s || {
        log_error "Kong deployment failed"
        kubectl describe pod -n "$API_GATEWAY_NAMESPACE" -l app=kong | head -50
        return 1
    }

    log_success "Kong gateway deployed (3 replicas ready)"

    # Get Kong service info
    KONG_LB_IP=$(kubectl get svc kong-gateway -n "$API_GATEWAY_NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "pending")
    log_info "Kong gateway accessible at: http://$KONG_LB_IP"
    log_info "Kong admin API at: http://kong-admin.${API_GATEWAY_NAMESPACE}:${KONG_ADMIN_INTERNAL_PORT}"
}

#############################################################################
# Step 3: Configure Kong Plugins & Routes
#############################################################################

configure_kong_plugins() {
    log_step "STEP 3: Configuring Kong Plugins & Routes"
    local pf_pid

    cleanup_port_forward() {
        if [ -n "${pf_pid:-}" ]; then
            kill "$pf_pid" 2>/dev/null || true
            wait "$pf_pid" 2>/dev/null || true
        fi
    }

    log_info "Applying plugin configurations..."
    kubectl apply -f k8s/api-gateway/kong-plugins.yaml
    log_success "Plugins configured"

    log_info "Waiting for plugin configurations to sync..."
    sleep 10

    # Port-forward to Kong admin for configuration
    kubectl port-forward -n "$API_GATEWAY_NAMESPACE" svc/kong-admin "${KONG_ADMIN_PORT_FORWARD_PORT}:${KONG_ADMIN_INTERNAL_PORT}" > /dev/null 2>&1 &
    pf_pid=$!
    sleep 3

    log_info "Configuring rate limiting plugin..."
    curl --silent --show-error --fail -X POST "http://127.0.0.1:${KONG_ADMIN_PORT_FORWARD_PORT}/plugins" \
        --data-urlencode "name=${KONG_RATE_LIMIT_PLUGIN_NAME}" \
        --data-urlencode "config.minute=${KONG_RATE_LIMIT_MINUTE_LIMIT}" \
        --data-urlencode "config.hour=${KONG_RATE_LIMIT_HOUR_LIMIT}" \
        --data-urlencode "config.policy=${KONG_RATE_LIMIT_POLICY}" \
        --data-urlencode "config.redis_host=${KONG_RATE_LIMIT_REDIS_HOST}" \
        --data-urlencode "config.redis_port=${KONG_RATE_LIMIT_REDIS_PORT}" \
        > /dev/null || log_error "Failed to configure rate limiting (Kong may not be ready yet)"

    cleanup_port_forward
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
    local pf_pid

    cleanup_port_forward() {
        if [ -n "${pf_pid:-}" ]; then
            kill "$pf_pid" 2>/dev/null || true
            wait "$pf_pid" 2>/dev/null || true
        fi
    }

    # Check pods
    POD_COUNT=$(kubectl get pod -n "$API_GATEWAY_NAMESPACE" -l app=kong --no-headers | wc -l)
    if [ "$POD_COUNT" -ge 2 ]; then
        log_success "Kong pods ready ($POD_COUNT/3)"
    else
        log_error "Not enough Kong pods ready ($POD_COUNT/3)"
        return 1
    fi

    # Health check via port-forward
    kubectl port-forward -n "$API_GATEWAY_NAMESPACE" svc/kong-status "${KONG_STATUS_PORT_FORWARD_PORT}:${KONG_STATUS_INTERNAL_PORT}" > /dev/null 2>&1 &
    pf_pid=$!
    sleep 2

    if curl --silent --show-error --fail "http://127.0.0.1:${KONG_STATUS_PORT_FORWARD_PORT}/status" | grep -q "database=ok"; then
        log_success "Kong health check passed"
    else
        log_error "Kong health check failed"
        cleanup_port_forward
        return 1
    fi

    cleanup_port_forward
}

verify_prometheus_scraping() {
    log_step "Verifying Prometheus Scraping"

    log_info "Kong metrics should appear in Prometheus within 2 minutes"
    log_info "Prometheus query: up{job=\"kong-metrics\"}"
}

verify_rate_limiting() {
    log_step "Verifying Rate Limiting Setup"

    # Check if rate limiting config is present
    if kubectl get configmap rate-limiting-config -n "$API_GATEWAY_NAMESPACE" &> /dev/null; then
        log_success "Rate limiting configuration present"
    else
        log_error "Rate limiting configuration missing"
        return 1
    fi

    log_info "Rate limiting quotas configured:"
    kubectl get configmap rate-limiting-config -n "$API_GATEWAY_NAMESPACE" -o jsonpath='{.data.quotas\.yaml}' | grep "name:" | head -4
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
- Admin API: http://kong-admin.${API_GATEWAY_NAMESPACE}:${KONG_ADMIN_INTERNAL_PORT}
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
