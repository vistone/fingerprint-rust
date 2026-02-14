#!/bin/bash

# Fingerprint API Kubernetes Deployment Script
# Usage: ./deploy.sh [staging|production] [apply|delete] [--dry-run]

set -e

ENVIRONMENT="${1:-staging}"
ACTION="${2:-apply}"
DRY_RUN="${3:-}"

# Validate inputs
if [[ ! "$ENVIRONMENT" =~ ^(staging|production)$ ]]; then
    echo "❌ Invalid environment: $ENVIRONMENT"
    echo "Usage: $0 [staging|production] [apply|delete] [--dry-run]"
    exit 1
fi

if [[ ! "$ACTION" =~ ^(apply|delete|preview)$ ]]; then
    echo "❌ Invalid action: $ACTION"
    echo "Usage: $0 [staging|production] [apply|delete|preview] [--dry-run]"
    exit 1
fi

# Construct kustomize path
KUSTOMIZE_PATH="./k8s/overlays/$ENVIRONMENT"

if [[ ! -d "$KUSTOMIZE_PATH" ]]; then
    echo "❌ Kustomize path not found: $KUSTOMIZE_PATH"
    exit 1
fi

echo "🚀 Fingerprint API Kubernetes Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Environment: $ENVIRONMENT"
echo "Action: $ACTION"
echo "Kustomize Path: $KUSTOMIZE_PATH"
echo ""

# Generate and display manifests
case "$ACTION" in
    preview)
        echo "📋 Preview of manifests to be deployed:"
        echo ""
        kubectl kustomize "$KUSTOMIZE_PATH"
        ;;
    
    apply)
        echo "📦 Building manifests from kustomize..."
        MANIFESTS=$(mktemp)
        kubectl kustomize "$KUSTOMIZE_PATH" > "$MANIFESTS"
        
        if [[ "$DRY_RUN" == "--dry-run" ]]; then
            echo "🔍 DRY-RUN: Manifests that would be deployed:"
            cat "$MANIFESTS"
            echo ""
            echo "🔍 Validating manifests..."
            kubectl apply -f "$MANIFESTS" --dry-run=client -o=json | jq . > /dev/null && echo "✅ Validation passed!"
        else
            echo "🔄 Waiting for user confirmation..."
            echo "⚠️  Press Enter to proceed with deployment or Ctrl+C to cancel"
            read
            
            echo "📤 Deploying to Kubernetes..."
            kubectl apply -f "$MANIFESTS"
            
            echo ""
            echo "⏳ Waiting for rollout..."
            kubectl rollout status deployment/fingerprint-api -n fingerprint --timeout=5m
            
            echo ""
            echo "✅ Deployment successful!"
            echo ""
            echo "📊 Deployment status:"
            kubectl get deployment -n fingerprint -o wide
            kubectl get pods -n fingerprint -o wide
            kubectl get svc -n fingerprint -o wide
        fi
        
        rm "$MANIFESTS"
        ;;
    
    delete)
        echo "⚠️  WARNING: This will delete the $ENVIRONMENT deployment"
        echo "Press Enter to proceed or Ctrl+C to cancel"
        read
        
        echo "🗑️ Deleting deployment..."
        kubectl kustomize "$KUSTOMIZE_PATH" | kubectl delete -f -
        
        echo "✅ Deletion complete"
        ;;
esac

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
