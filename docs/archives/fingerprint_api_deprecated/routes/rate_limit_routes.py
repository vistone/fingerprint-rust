"""
Rate Limiting API Routes

Endpoints for rate limiting management and metrics exposure.
"""

from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import PlainTextResponse, JSONResponse
from typing import Optional
import json

from fingerprint_api.services.rate_limit_service import RateLimitService, QuotaTier
from fingerprint_api.schemas.rate_limit import (
    RateLimitResponse,
    QuotaInfo,
    HealthCheck,
    PrometheusMetrics,
)

router = APIRouter(prefix="/api/v1/rate-limit", tags=["rate-limiting"])


def get_rate_limit_service() -> RateLimitService:
    """Get singleton rate limit service instance."""
    # In real application, would be from app.state or dependency injection
    return RateLimitService()


@router.get(
    "/status",
    response_model=dict,
    summary="Rate Limit Service Status",
    description="Get current rate limiting service status",
)
async def get_status():
    """Get rate limiting service status."""
    service = get_rate_limit_service()
    metrics = service.get_metrics()

    return {
        "status": "operational",
        "redis_enabled": service.redis_enabled,
        "metrics": metrics,
        "timestamp": int(
            __import__("time").time()
        ),  # Unix timestamp
    }


@router.get(
    "/health",
    response_model=HealthCheck,
    summary="Health Check",
    description="Check rate limiting service health",
)
async def health_check():
    """Health check endpoint."""
    service = get_rate_limit_service()
    uptime_seconds = __import__("time").time()  # Simplified

    return HealthCheck(
        status="healthy",
        timestamp=__import__("datetime").datetime.utcnow(),
        redis_connected=service.redis_enabled,
        metrics_available=service.metrics_enabled,
        uptime_seconds=uptime_seconds,
    )


@router.get(
    "/metrics",
    response_class=PlainTextResponse,
    summary="Prometheus Metrics",
    description="Export metrics in Prometheus text format",
)
async def get_prometheus_metrics():
    """Get metrics in Prometheus format."""
    service = get_rate_limit_service()
    return service.get_prometheus_metrics()


@router.get(
    "/metrics/json",
    response_class=JSONResponse,
    summary="Metrics (JSON)",
    description="Get metrics in JSON format",
)
async def get_json_metrics():
    """Get metrics in JSON format."""
    service = get_rate_limit_service()
    metrics = service.get_metrics()

    return {
        "metrics": metrics,
        "timestamp": __import__("time").time(),
        "prometheus_format": service.get_prometheus_metrics(),
    }


@router.get(
    "/quota/{user_id}",
    response_model=dict,
    summary="Get User Quota",
    description="Get current quota information for user",
)
async def get_user_quota(user_id: str, request: Request):
    """Get user's current quota status."""
    service = get_rate_limit_service()

    if user_id not in service.user_quotas:
        raise HTTPException(status_code=404, detail="User not found")

    quota = service.user_quotas[user_id]

    return {
        "user_id": user_id,
        "tier": quota.tier.value,
        "minute_limit": service._get_minute_limit(quota.tier),
        "monthly_quota": service.tier_limits[quota.tier][1],
        "available_tokens": quota.available_tokens,
        "total_requests": quota.total_requests,
        "month_requests": quota.month_requests,
        "last_refill": quota.last_refill,
    }


@router.post(
    "/quota/{user_id}/reset",
    summary="Reset User Quota",
    description="Reset quota for user (admin only)",
)
async def reset_user_quota(user_id: str, request: Request):
    """Reset user quota (for testing/admin)."""
    # In production, would check admin authorization
    service = get_rate_limit_service()
    service.clear_user_quota(user_id)

    return {
        "message": f"Quota reset for user {user_id}",
        "timestamp": __import__("time").time(),
    }


@router.post(
    "/check",
    response_model=dict,
    summary="Check Rate Limit",
    description="Check if request would be allowed",
)
async def check_rate_limit(request: Request):
    """Check if a hypothetical request would be rate limited."""
    service = get_rate_limit_service()

    # Parse request
    body = await request.json()
    user_id = body.get("user_id")
    user_tier_str = body.get("user_tier", "free")
    endpoint = body.get("endpoint", "/identify")

    # Convert tier string to enum
    try:
        user_tier = QuotaTier[user_tier_str.capitalize()]
    except KeyError:
        raise HTTPException(
            status_code=400,
            detail=f"Invalid tier: {user_tier_str}",
        )

    # Check limit
    try:
        response = service.check_limit(
            user_id=user_id,
            user_tier=user_tier,
            endpoint=endpoint,
            client_ip=request.client.host if request.client else None,
        )

        return {
            "allowed": True,
            "rate_limit_response": response,
        }
    except Exception as e:
        return {
            "allowed": False,
            "error": str(e),
        }


@router.get(
    "/tiers",
    response_model=dict,
    summary="Get Tier Definitions",
    description="Get rate limiting tiers and their limits",
)
async def get_tier_definitions():
    """Get all tier definitions."""
    service = get_rate_limit_service()

    tiers = {}
    for tier, (minute_limit, monthly_quota) in service.tier_limits.items():
        tiers[tier.value] = {
            "minute_limit": minute_limit if minute_limit is not None else float("inf"),
            "monthly_quota": monthly_quota if monthly_quota is not None else float("inf"),
            "burst_multiplier": 1.5,
            "description": {
                QuotaTier.Free: "Free tier - limited to 100 requests/minute",
                QuotaTier.Pro: "Pro tier - 1000 requests/minute",
                QuotaTier.Enterprise: "Enterprise tier - unlimited",
                QuotaTier.Partner: "Partner tier - unlimited",
            }[tier],
        }

    return {
        "tiers": tiers,
        "default_ip_limit": 30,
        "burst_multiplier": 1.5,
        "minute_window": 60,
    }


@router.get(
    "/endpoints",
    response_model=dict,
    summary="Get Endpoint Configurations",
    description="Get cost configurations for all endpoints",
)
async def get_endpoint_configs():
    """Get endpoint cost configurations."""
    service = get_rate_limit_service()

    endpoints = {}
    for path, config in service.endpoints.items():
        endpoints[path] = {
            "cost": config.cost,
            "require_auth": config.require_auth,
            "max_burst": config.max_burst,
        }

    # Add defaults for common endpoints
    defaults = {
        "/identify": {"cost": 1.0, "require_auth": True, "max_burst": 1.5},
        "/compare": {"cost": 2.0, "require_auth": True, "max_burst": 1.5},
        "/batch": {"cost": 1.0, "require_auth": True, "max_burst": 1.5},
        "/health": {"cost": 0.0, "require_auth": False, "max_burst": 1.5},
    }

    for path, config in defaults.items():
        if path not in endpoints:
            endpoints[path] = config

    return {"endpoints": endpoints}


def setup_rate_limit_routes(app):
    """
    Setup rate limiting routes on FastAPI app.
    
    Usage:
    ```python
    from fastapi import FastAPI
    from fingerprint_api.routes.rate_limit_routes import setup_rate_limit_routes
    
    app = FastAPI()
    setup_rate_limit_routes(app)
    ```
    """
    app.include_router(router)
