#!/bin/bash
#
# Kong API Gateway Configuration Script
# Sets up routes, services, and plugins for fingerprint API
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default values
NAMESPACE="api-gateway"
KONG_ADMIN_URL=""
SKIP_WAIT=false

# Usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -n, --namespace    Kubernetes namespace [default: api-gateway]"
    echo "  -u, --url          Kong Admin API URL (auto-detected if not provided)"
    echo "  -s, --skip-wait    Skip waiting for Kong to be ready"
    echo "  -h, --help         Show this help message"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        -u|--url)
            KONG_ADMIN_URL="$2"
            shift 2
            ;;
        -s|--skip-wait)
            SKIP_WAIT=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
    esac
done

# Auto-detect Kong Admin URL if not provided
if [ -z "$KONG_ADMIN_URL" ]; then
    # Try to get from port-forward or service
    if kubectl get svc kong-admin -n "$NAMESPACE" &>/dev/null; then
        # Check if port-forward is already running
        if lsof -Pi :8001 -sTCP:LISTEN -t >/dev/null 2>&1; then
            KONG_ADMIN_URL="http://localhost:8001"
        else
            echo -e "${YELLOW}Starting port-forward to Kong Admin API...${NC}"
            kubectl port-forward svc/kong-admin 8001:8001 -n "$NAMESPACE" &
            PORT_FORWARD_PID=$!
            sleep 3
            KONG_ADMIN_URL="http://localhost:8001"
            
            # Cleanup port-forward on exit
            trap "kill $PORT_FORWARD_PID 2>/dev/null || true" EXIT
        fi
    else
        echo -e "${RED}Cannot find Kong Admin service in namespace $NAMESPACE${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  Kong API Gateway Configuration${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo "Namespace: $NAMESPACE"
echo "Admin URL: $KONG_ADMIN_URL"
echo ""

# Wait for Kong to be ready
if [ "$SKIP_WAIT" = false ]; then
    echo -e "${YELLOW}Waiting for Kong Admin API...${NC}"
    for i in {1..30}; do
        if curl -s "$KONG_ADMIN_URL/status" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Kong is ready${NC}"
            break
        fi
        echo "  Attempt $i/30..."
        sleep 2
    done
fi

# Helper function to create/update Kong resources
kong_upsert() {
    local endpoint=$1
    local data=$2
    local name=$3
    
    # Check if resource exists
    if curl -s "$KONG_ADMIN_URL/$endpoint/$name" | grep -q '"id"'; then
        echo "  Updating $name..."
        curl -s -X PATCH "$KONG_ADMIN_URL/$endpoint/$name" \
            -H "Content-Type: application/json" \
            -d "$data" > /dev/null
    else
        echo "  Creating $name..."
        curl -s -X POST "$KONG_ADMIN_URL/$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data" > /dev/null
    fi
}

echo -e "${YELLOW}Configuring Services...${NC}"

# Create fingerprint API service
kong_upsert "services" '{
    "name": "fingerprint-api",
    "url": "http://fingerprint-api.fingerprint-api.svc.cluster.local:3000",
    "connect_timeout": 5000,
    "write_timeout": 30000,
    "read_timeout": 30000,
    "retries": 3
}' "fingerprint-api"

echo -e "${GREEN}✓ Services configured${NC}"
echo ""

echo -e "${YELLOW}Configuring Routes...${NC}"

# Main fingerprint route
kong_upsert "routes" '{
    "name": "fingerprint-route",
    "service": {"name": "fingerprint-api"},
    "paths": ["/fingerprint", "/v1"],
    "methods": ["GET", "POST", "PUT", "DELETE"],
    "strip_path": false,
    "preserve_host": false
}' "fingerprint-route"

# Health check route (no auth)
kong_upsert "routes" '{
    "name": "health-route",
    "service": {"name": "fingerprint-api"},
    "paths": ["/health", "/status"],
    "methods": ["GET"],
    "strip_path": false
}' "health-route"

echo -e "${GREEN}✓ Routes configured${NC}"
echo ""

echo -e "${YELLOW}Configuring Plugins...${NC}"

# Rate Limiting Plugin (with Redis)
curl -s -X POST "$KONG_ADMIN_URL/plugins" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "rate-limiting",
        "config": {
            "minute": 100,
            "hour": 3600,
            "limit_by": "consumer",
            "policy": "redis",
            "redis_host": "redis-cluster.caching.svc.cluster.local",
            "redis_port": 6379,
            "fault_tolerant": true,
            "hide_client_headers": false
        }
    }' > /dev/null
echo "  ✓ Rate limiting plugin enabled"

# CORS Plugin
curl -s -X POST "$KONG_ADMIN_URL/plugins" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "cors",
        "config": {
            "origins": ["*"],
            "methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
            "headers": ["Content-Type", "Authorization", "X-API-Key"],
            "exposed_headers": ["X-RateLimit-Limit", "X-RateLimit-Remaining"],
            "max_age": 86400,
            "credentials": true
        }
    }' > /dev/null
echo "  ✓ CORS plugin enabled"

# Key Auth Plugin (for fingerprint routes only)
curl -s -X POST "$KONG_ADMIN_URL/plugins" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "key-auth",
        "route": {"name": "fingerprint-route"},
        "config": {
            "key_names": ["apikey", "x-api-key"],
            "hide_credentials": true
        }
    }' > /dev/null
echo "  ✓ Key authentication plugin enabled"

# Request Transformer (add headers)
curl -s -X POST "$KONG_ADMIN_URL/plugins" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "request-transformer",
        "config": {
            "add": {
                "headers": ["X-Forwarded-Proto:https"]
            }
        }
    }' > /dev/null
echo "  ✓ Request transformer plugin enabled"

echo -e "${GREEN}✓ Plugins configured${NC}"
echo ""

echo -e "${YELLOW}Creating Default Consumer...${NC}"

# Create a default consumer for testing
curl -s -X POST "$KONG_ADMIN_URL/consumers" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "default-user",
        "custom_id": "default-user-001"
    }' > /dev/null

# Create API key for default consumer
API_KEY=$(curl -s -X POST "$KONG_ADMIN_URL/consumers/default-user/key-auth" \
    -H "Content-Type: application/json" \
    -d '{}' | grep -o '"key":"[^"]*"' | cut -d'"' -f4)

echo -e "${GREEN}✓ Default consumer created${NC}"
echo ""

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  Configuration Complete${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo "Default API Key: $API_KEY"
echo ""
echo "Test Commands:"
echo "  curl -H \"apikey: $API_KEY\" http://localhost:8080/fingerprint/health"
echo "  curl -H \"apikey: $API_KEY\" http://localhost:8080/v1/users/me"
echo ""
echo "To view metrics:"
echo "  curl http://localhost:8001/metrics"
echo ""
