#!/bin/bash
#
# API Gateway Deployment Script
# Deploys Kong API Gateway with Redis for distributed rate limiting
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
ENVIRONMENT="development"
NAMESPACE="api-gateway"
ACTION="apply"

# Usage information
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -e, --environment    Environment (development|staging|production) [default: development]"
    echo "  -n, --namespace      Kubernetes namespace [default: api-gateway]"
    echo "  -d, --delete         Delete resources instead of applying"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 -e development                    # Deploy to development"
    echo "  $0 -e production -n api-gateway-prod # Deploy to production"
    echo "  $0 -e development -d                 # Delete development deployment"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -n|--namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        -d|--delete)
            ACTION="delete"
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

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(development|staging|production)$ ]]; then
    echo -e "${RED}Invalid environment: $ENVIRONMENT${NC}"
    echo "Valid environments: development, staging, production"
    exit 1
fi

# Set namespace based on environment
if [ "$NAMESPACE" == "api-gateway" ]; then
    case $ENVIRONMENT in
        development)
            NAMESPACE="api-gateway-dev"
            ;;
        staging)
            NAMESPACE="api-gateway-staging"
            ;;
        production)
            NAMESPACE="api-gateway"
            ;;
    esac
fi

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  API Gateway Deployment${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo "Environment: $ENVIRONMENT"
echo "Namespace:   $NAMESPACE"
echo "Action:      $ACTION"
echo ""

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}kubectl is not installed${NC}"
    exit 1
fi

if ! command -v kustomize &> /dev/null && ! kubectl kustomize &> /dev/null; then
    echo -e "${RED}kustomize is not installed${NC}"
    exit 1
fi

# Check cluster connectivity
if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}Cannot connect to Kubernetes cluster${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Prerequisites met${NC}"
echo ""

# Function to apply or delete resources
apply_resources() {
    local overlay=$1
    local ns=$2
    
    if [ "$ACTION" == "delete" ]; then
        echo -e "${YELLOW}Deleting resources from $overlay...${NC}"
        kubectl delete -k "$overlay" --ignore-not-found=true
        kubectl delete namespace "$ns" --ignore-not-found=true
        echo -e "${GREEN}✓ Resources deleted${NC}"
    else
        echo -e "${YELLOW}Applying resources from $overlay...${NC}"
        
        # Create namespace if it doesn't exist
        kubectl create namespace "$ns" --dry-run=client -o yaml | kubectl apply -f -
        
        # Apply kustomization
        kubectl apply -k "$overlay"
        
        echo -e "${GREEN}✓ Resources applied${NC}"
    fi
}

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OVERLAY_PATH="$SCRIPT_DIR/overlays/$ENVIRONMENT"

# Apply resources
apply_resources "$OVERLAY_PATH" "$NAMESPACE"

# Wait for deployments if applying
if [ "$ACTION" == "apply" ]; then
    echo ""
    echo -e "${YELLOW}Waiting for deployments to be ready...${NC}"
    
    # Wait for Redis
    echo "Waiting for Redis..."
    kubectl wait --for=condition=ready pod -l app=redis -n "$NAMESPACE" --timeout=120s || true
    
    # Wait for PostgreSQL
    echo "Waiting for PostgreSQL..."
    kubectl wait --for=condition=ready pod -l app=kong-postgres -n "$NAMESPACE" --timeout=120s || true
    
    # Wait for Kong migrations
    echo "Waiting for Kong migrations..."
    kubectl wait --for=condition=complete job/kong-migrations -n "$NAMESPACE" --timeout=180s || true
    
    # Wait for Kong
    echo "Waiting for Kong..."
    kubectl wait --for=condition=available deployment/kong -n "$NAMESPACE" --timeout=180s || true
    
    echo -e "${GREEN}✓ All deployments ready${NC}"
fi

# Show status
echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  Deployment Status${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""

echo "Pods:"
kubectl get pods -n "$NAMESPACE"
echo ""

echo "Services:"
kubectl get services -n "$NAMESPACE"
echo ""

if [ "$ACTION" == "apply" ]; then
    echo -e "${GREEN}============================================${NC}"
    echo -e "${GREEN}  Access Information${NC}"
    echo -e "${GREEN}============================================${NC}"
    echo ""
    echo "Kong Gateway:"
    echo "  kubectl port-forward svc/kong-gateway 8080:80 -n $NAMESPACE"
    echo "  http://localhost:8080"
    echo ""
    echo "Kong Admin API:"
    echo "  kubectl port-forward svc/kong-admin 8001:8001 -n $NAMESPACE"
    echo "  http://localhost:8001"
    echo ""
    echo "To configure routes and plugins:"
    echo "  ./scripts/configure-kong.sh -n $NAMESPACE"
    echo ""
fi

echo -e "${GREEN}Done!${NC}"
