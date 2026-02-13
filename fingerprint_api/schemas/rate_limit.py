"""
Rate Limiting Data Schemas

Pydantic models for rate limiting requests and responses.
"""

from pydantic import BaseModel, Field
from enum import Enum
from typing import Optional
from datetime import datetime


class QuotaTierEnum(str, Enum):
    """User quota tiers."""

    FREE = "free"
    PRO = "pro"
    ENTERPRISE = "enterprise"
    PARTNER = "partner"


class RateLimitResponse(BaseModel):
    """Rate limit check response."""

    allowed: bool = Field(..., description="Whether request is allowed")
    remaining: int = Field(..., description="Remaining requests in current minute")
    reset_at: int = Field(..., description="Unix timestamp when limit resets")
    tier: QuotaTierEnum = Field(..., description="User's quota tier")
    monthly_remaining: int = Field(..., description="Remaining requests in current month")


class RateLimitError(BaseModel):
    """Rate limit error response (429)."""

    error: str = Field(..., description="Error code")
    message: str = Field(..., description="Human-readable error message")
    reset_at: int = Field(..., description="Unix timestamp when limit resets")
    retry_after: int = Field(60, description="Seconds to wait before retry")


class QuotaInfo(BaseModel):
    """User quota information."""

    user_id: str = Field(..., description="User ID or API key")
    tier: QuotaTierEnum = Field(..., description="User's quota tier")
    minute_limit: int = Field(..., description="Requests per minute limit")
    monthly_quota: int = Field(..., description="Total requests per month limit")
    current_minute_requests: int = Field(0, description="Requests in current minute")
    current_month_requests: int = Field(0, description="Requests in current month")
    last_reset: datetime = Field(..., description="Last minute reset timestamp")
    month_start: datetime = Field(..., description="Current month start timestamp")


class EndpointConfig(BaseModel):
    """Configuration for an API endpoint rate limiting."""

    endpoint: str = Field(..., description="API endpoint path")
    cost: float = Field(1.0, description="Token cost multiplier (1.0 = normal)")
    require_auth: bool = Field(True, description="Whether authentication is required")
    max_burst: float = Field(1.5, description="Burst multiplier for token bucket")


class RateLimitConfig(BaseModel):
    """Rate limiting configuration."""

    free_tier_minute_limit: int = Field(100, description="Free tier req/min")
    free_tier_monthly_quota: int = Field(50000, description="Free tier monthly quota")
    pro_tier_minute_limit: int = Field(1000, description="Pro tier req/min")
    pro_tier_monthly_quota: int = Field(1000000, description="Pro tier monthly quota")
    enterprise_tier_minute_limit: Optional[int] = Field(
        None, description="Enterprise tier limit (None = unlimited)"
    )
    enterprise_tier_monthly_quota: Optional[int] = Field(
        None, description="Enterprise tier quota (None = unlimited)"
    )
    ip_fallback_limit: int = Field(30, description="IP-based limit req/min")
    ip_whitelist: list = Field(
        default_factory=list, description="IP whitelist (CIDR notation)"
    )
    endpoints: list = Field(
        default_factory=list, description="List of EndpointConfig"
    )


class MetricsSnapshot(BaseModel):
    """Metrics snapshot for monitoring."""

    timestamp: datetime = Field(..., description="Snapshot timestamp")
    total_requests: int = Field(0, description="Total requests processed")
    total_rejected: int = Field(0, description="Total requests rejected")
    cache_hits: int = Field(0, description="Cache hits")
    cache_misses: int = Field(0, description="Cache misses")
    rejection_rate: float = Field(0.0, description="Rejection rate percentage")
    cache_hit_ratio: float = Field(0.0, description="Cache hit ratio")


class TierMetricsSnapshot(BaseModel):
    """Per-tier metrics snapshot."""

    tier: QuotaTierEnum = Field(..., description="Quota tier")
    user_count: int = Field(0, description="Active users in tier")
    total_requests: int = Field(0, description="Requests from tier")
    rejected_requests: int = Field(0, description="Rejected requests from tier")
    avg_requests_per_user: float = Field(0.0, description="Average requests per user")


class PrometheusMetrics(BaseModel):
    """Prometheus metrics export model."""

    text_format: str = Field(..., description="Prometheus text format")
    json_format: dict = Field(..., description="JSON format metrics")


class HealthCheck(BaseModel):
    """Service health check response."""

    status: str = Field(..., description="Service status (healthy, degraded, down)")
    timestamp: datetime = Field(..., description="Check timestamp")
    redis_connected: bool = Field(..., description="Redis connection status")
    metrics_available: bool = Field(..., description="Metrics collection status")
    uptime_seconds: float = Field(..., description="Service uptime")
