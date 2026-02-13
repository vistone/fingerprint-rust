"""
FastAPI Rate Limiting Middleware

Integrates with Phase 9.4 Kong API Gateway and Rust rate limiting service.
Enforces user quotas and generates RFC 6585 compliant responses.
"""

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import JSONResponse
from starlette.requests import Request
import time
import json
from typing import Optional, Dict, Tuple
from enum import Enum
import os

from fingerprint_api.services.rate_limit_service import (
    RateLimitService,
    QuotaTier,
    RateLimitError,
)


class RateLimitMiddleware(BaseHTTPMiddleware):
    """
    Middleware that enforces rate limiting on incoming requests.
    
    Features:
    - Per-user quota enforcement
    - IP-based fallback for unauthenticated requests
    - RFC 6585 compliant 429 responses
    - Prometheus compatible metrics
    """

    def __init__(self, app, rate_limit_service: Optional[RateLimitService] = None):
        """
        Initialize middleware.
        
        Args:
            app: FastAPI application
            rate_limit_service: Rate limiter service instance (created if None)
        """
        super().__init__(app)
        self.rate_limit_service = rate_limit_service or RateLimitService()
        
        # Exempt paths from rate limiting
        self.exempt_paths = {
            "/health",
            "/metrics",
            "/docs",
            "/redoc",
            "/openapi.json",
        }

    async def dispatch(self, request: Request, call_next):
        """
        Process request through rate limiter.
        """
        # Skip rate limiting for exempt paths
        if request.url.path in self.exempt_paths:
            return await call_next(request)

        # Extract user information
        user_id = self._extract_user_id(request)
        user_tier = self._extract_user_tier(request)
        endpoint = request.url.path
        client_ip = self._extract_client_ip(request)

        # Check rate limit
        try:
            rate_limit_response = self.rate_limit_service.check_limit(
                user_id=user_id,
                user_tier=user_tier,
                endpoint=endpoint,
                client_ip=client_ip,
            )
        except RateLimitError as e:
            return self._handle_rate_limit_error(e, request)

        # Request allowed - proceed
        start_time = time.time()
        response = await call_next(request)
        duration_ms = (time.time() - start_time) * 1000

        # Add rate limit response headers (RFC 6585 compliant)
        response.headers["X-RateLimit-Remaining"] = str(rate_limit_response["remaining"])
        response.headers["X-RateLimit-Reset"] = str(rate_limit_response["reset_at"])
        response.headers["X-Quota-Tier"] = str(rate_limit_response["tier"]).lower()
        response.headers["X-Quota-Monthly-Remaining"] = str(
            rate_limit_response["monthly_remaining"]
        )

        # Add performance metrics
        response.headers["X-Response-Time"] = f"{duration_ms:.0f}ms"

        return response

    def _extract_user_id(self, request: Request) -> Optional[str]:
        """Extract user ID from request."""
        # Try from X-API-Key header
        api_key = request.headers.get("X-API-Key")
        if api_key:
            return api_key

        # Try from Authorization header (Bearer token)
        auth_header = request.headers.get("Authorization", "")
        if auth_header.startswith("Bearer "):
            return auth_header[7:]

        # Try from query parameter
        return request.query_params.get("api_key")

    def _extract_user_tier(self, request: Request) -> QuotaTier:
        """
        Extract user tier from request.
        
        In production, this would query a database based on user_id.
        """
        # Check for explicit tier header (for testing)
        tier_header = request.headers.get("X-Quota-Tier", "free").lower()
        
        tier_map = {
            "free": QuotaTier.Free,
            "pro": QuotaTier.Pro,
            "enterprise": QuotaTier.Enterprise,
            "partner": QuotaTier.Partner,
        }

        return tier_map.get(tier_header, QuotaTier.Free)

    def _extract_client_ip(self, request: Request) -> Optional[str]:
        """Extract client IP from request."""
        # Check X-Forwarded-For (from Kong gateway)
        forwarded_for = request.headers.get("X-Forwarded-For")
        if forwarded_for:
            return forwarded_for.split(",")[0].strip()

        # Check X-Real-IP (from nginx)
        real_ip = request.headers.get("X-Real-IP")
        if real_ip:
            return real_ip

        # Fall back to client socket
        if request.client:
            return request.client.host

        return None

    def _handle_rate_limit_error(
        self, error: RateLimitError, request: Request
    ) -> JSONResponse:
        """Generate 429 Too Many Requests response."""
        now = int(time.time())

        if error.error_type == "monthly_quota_exceeded":
            return JSONResponse(
                status_code=429,
                content={
                    "error": "monthly_quota_exceeded",
                    "message": "Monthly quota exceeded. Please upgrade your plan.",
                    "reset_at": error.reset_at or now + (30 * 86400),
                },
                headers={
                    "Retry-After": str(error.retry_after or 86400),
                    "X-RateLimit-Reset": str(error.reset_at or now + (30 * 86400)),
                    "Content-Type": "application/json",
                },
            )
        else:  # rate_limit_exceeded
            return JSONResponse(
                status_code=429,
                content={
                    "error": "rate_limit_exceeded",
                    "message": "Too many requests. Please try again later.",
                    "reset_at": now + 60,
                },
                headers={
                    "Retry-After": str(error.retry_after or 60),
                    "X-RateLimit-Reset": str(now + 60),
                    "Content-Type": "application/json",
                },
            )


class CORSRateLimitMiddleware(BaseHTTPMiddleware):
    """
    Middleware to handle CORS preflight OPTIONS requests without rate limiting.
    Install before RateLimitMiddleware.
    """

    async def dispatch(self, request: Request, call_next):
        """Allow CORS preflight without rate limiting."""
        if request.method == "OPTIONS":
            return JSONResponse(
                status_code=200,
                content={},
                headers={
                    "Access-Control-Allow-Origin": "*",
                    "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
                    "Access-Control-Allow-Headers": "Content-Type, X-API-Key, Authorization",
                    "Access-Control-Max-Age": "3600",
                },
            )

        return await call_next(request)


class MetricsCollectionMiddleware(BaseHTTPMiddleware):
    """
    Middleware to collect request metrics for Prometheus.
    """

    def __init__(self, app):
        """Initialize metrics middleware."""
        super().__init__(app)
        self.request_count = 0
        self.rate_limit_rejections = 0
        self.total_response_time_ms = 0.0

    async def dispatch(self, request: Request, call_next):
        """Collect metrics from each request."""
        start_time = time.time()
        self.request_count += 1

        response = await call_next(request)

        # Count rate limit rejections
        if response.status_code == 429:
            self.rate_limit_rejections += 1

        duration_ms = (time.time() - start_time) * 1000
        self.total_response_time_ms += duration_ms

        return response

    def get_metrics(self) -> Dict:
        """Get collected metrics."""
        avg_response_time = (
            self.total_response_time_ms / self.request_count
            if self.request_count > 0
            else 0
        )

        return {
            "total_requests": self.request_count,
            "rate_limit_rejections": self.rate_limit_rejections,
            "rejection_rate": (
                self.rate_limit_rejections / self.request_count
                if self.request_count > 0
                else 0
            ),
            "avg_response_time_ms": avg_response_time,
        }


def setup_rate_limiting(app, rate_limit_service: Optional[RateLimitService] = None):
    """
    Setup rate limiting middleware for FastAPI app.
    
    Usage in FastAPI app:
    ```python
    from fastapi import FastAPI
    from fingerprint_api.middleware.rate_limiter import setup_rate_limiting
    
    app = FastAPI()
    setup_rate_limiting(app)
    
    @app.get("/identify")
    async def identify(request: Request):
        ...
    ```
    """
    # Add CORS handling first (run first in middleware stack)
    app.add_middleware(CORSRateLimitMiddleware)

    # Add metrics collection
    app.add_middleware(MetricsCollectionMiddleware)

    # Add rate limiting (run last, closest to handler)
    app.add_middleware(RateLimitMiddleware, rate_limit_service=rate_limit_service)
