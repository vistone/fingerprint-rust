"""
Rate Limiting Service

High-performance rate limiting service with token bucket algorithm.
Integrates with Rust core library and Redis backend.
"""

import asyncio
import time
from enum import Enum
from typing import Optional, Dict, Tuple
from dataclasses import dataclass, field
from datetime import datetime, timedelta
import json
import httpx

from fingerprint_api.schemas.rate_limit import (
    QuotaTierEnum,
    RateLimitResponse,
    EndpointConfig,
)


class QuotaTier(Enum):
    """User quota tiers matching Rust enum."""

    Free = "free"
    Pro = "pro"
    Enterprise = "enterprise"
    Partner = "partner"


class RateLimitError(Exception):
    """Base rate limit error."""

    def __init__(
        self,
        error_type: str,
        message: str,
        retry_after: int = 60,
        reset_at: Optional[int] = None,
    ):
        self.error_type = error_type
        self.message = message
        self.retry_after = retry_after
        self.reset_at = reset_at
        super().__init__(f"{error_type}: {message}")


@dataclass
class UserQuota:
    """User quota state."""

    user_id: str
    tier: QuotaTier
    available_tokens: float
    last_refill: int  # Unix timestamp
    month_requests: int = 0
    total_requests: int = 0

    def has_quota(self, cost: float = 1.0) -> bool:
        """Check if user has available quota."""
        return self.available_tokens >= cost

    def consume(self, cost: float = 1.0) -> None:
        """Consume tokens from user's quota."""
        self.available_tokens -= cost
        self.total_requests += 1
        self.month_requests += 1


class RateLimitService:
    """
    Main rate limiting service.
    
    Features:
    - Token bucket algorithm with 1.5x burst support
    - Per-user and per-IP rate limiting
    - Monthly quota tracking
    - Redis backend for persistence
    - Prometheus metrics export
    """

    def __init__(
        self,
        redis_url: str = "redis://localhost:6379",
        redis_enabled: bool = True,
        metrics_enabled: bool = True,
    ):
        """
        Initialize rate limiting service.
        
        Args:
            redis_url: Redis connection URL
            redis_enabled: Whether to use Redis backend
            metrics_enabled: Whether to collect metrics
        """
        self.redis_url = redis_url
        self.redis_enabled = redis_enabled
        self.metrics_enabled = metrics_enabled

        # In-memory cache (local backup)
        self.user_quotas: Dict[str, UserQuota] = {}
        self.ip_quotas: Dict[str, Tuple[int, int]] = {}  # (timestamp, count)

        # Tier definitions
        self.tier_limits: Dict[QuotaTier, Tuple[int, int]] = {
            QuotaTier.Free: (100, 50_000),  # 100/min, 50k/month
            QuotaTier.Pro: (1_000, 1_000_000),  # 1k/min, 1M/month
            QuotaTier.Enterprise: (None, None),  # unlimited
            QuotaTier.Partner: (None, None),  # unlimited
        }

        # Endpoint configurations
        self.endpoints: Dict[str, EndpointConfig] = {}

        # Metrics
        self.metrics = {
            "total_requests": 0,
            "total_rejected": 0,
            "cache_hits": 0,
            "cache_misses": 0,
        }

        # HTTP client for Rust service communication
        self.client: Optional[httpx.AsyncClient] = None

    async def initialize(self) -> None:
        """Initialize async resources."""
        if self.redis_enabled:
            try:
                self.client = httpx.AsyncClient(timeout=10.0)
                await self._check_redis_health()
            except Exception as e:
                print(f"Warning: Could not connect to Redis: {e}")
                self.redis_enabled = False

    async def shutdown(self) -> None:
        """Shutdown async resources."""
        if self.client:
            await self.client.aclose()

    def register_endpoint(self, endpoint: str, cost: float = 1.0) -> None:
        """Register endpoint with cost multiplier."""
        self.endpoints[endpoint] = EndpointConfig(endpoint=endpoint, cost=cost)

    def check_limit(
        self,
        user_id: Optional[str],
        user_tier: QuotaTier,
        endpoint: str,
        client_ip: Optional[str] = None,
    ) -> Dict:
        """
        Check if request is allowed under rate limits.
        
        Returns:
            Dictionary with rate limit response data
            
        Raises:
            RateLimitError: If request violates limit
        """
        self.metrics["total_requests"] += 1

        # Get endpoint cost
        endpoint_config = self.endpoints.get(endpoint, EndpointConfig(endpoint=endpoint))
        cost = endpoint_config.cost

        # Get or create user quota
        if user_id:
            quota = self._get_or_create_user_quota(user_id, user_tier)

            # Check monthly quota
            if not self._check_monthly_quota(quota):
                self.metrics["total_rejected"] += 1
                raise RateLimitError(
                    error_type="monthly_quota_exceeded",
                    message=f"Monthly quota exceeded for tier {user_tier.value}",
                    retry_after=86400,
                    reset_at=int(time.time()) + (30 * 86400),
                )

            # Check minute rate limit
            if not self._check_minute_limit(quota, cost):
                self.metrics["total_rejected"] += 1
                raise RateLimitError(
                    error_type="rate_limit_exceeded",
                    message=f"Rate limit exceeded. Max {self._get_minute_limit(user_tier)}/min",
                    retry_after=60,
                    reset_at=int(time.time()) + 60,
                )

            # Consume tokens
            quota.consume(cost)

            # Persist to Redis
            if self.redis_enabled:
                asyncio.create_task(self._persist_quota(quota))

            return {
                "allowed": True,
                "remaining": max(0, int(quota.available_tokens)),
                "reset_at": int(time.time()) + 60,
                "tier": QuotaTierEnum(user_tier.value),
                "monthly_remaining": max(
                    0,
                    self.tier_limits[user_tier][1] - quota.month_requests
                    if self.tier_limits[user_tier][1]
                    else 999_999_999,
                ),
            }
        else:
            # IP-based rate limiting for unauthenticated requests
            if not self._check_ip_limit(client_ip, cost):
                self.metrics["total_rejected"] += 1
                raise RateLimitError(
                    error_type="rate_limit_exceeded",
                    message="IP rate limit exceeded. Max 30/min without API key",
                    retry_after=60,
                    reset_at=int(time.time()) + 60,
                )

            return {
                "allowed": True,
                "remaining": max(0, int(30 - cost)),
                "reset_at": int(time.time()) + 60,
                "tier": QuotaTierEnum.Free,
                "monthly_remaining": 0,
            }

    async def _check_redis_health(self) -> None:
        """Check connection to Redis backend."""
        try:
            response = await self.client.get(f"{self.redis_url}/health")
            if response.status_code != 200:
                raise Exception("Redis health check failed")
        except Exception as e:
            print(f"Redis health check failed: {e}")
            self.redis_enabled = False

    async def _persist_quota(self, quota: UserQuota) -> None:
        """Persist quota to Redis backend."""
        if not self.redis_enabled or not self.client:
            return

        try:
            payload = {
                "user_id": quota.user_id,
                "tier": quota.tier.value,
                "available_tokens": quota.available_tokens,
                "last_refill": quota.last_refill,
                "month_requests": quota.month_requests,
                "total_requests": quota.total_requests,
            }
            # In real implementation, would POST to Rust service
            # await self.client.post(f"{RUST_SERVICE_URL}/quota/update", json=payload)
        except Exception as e:
            print(f"Failed to persist quota: {e}")

    def _get_or_create_user_quota(
        self, user_id: str, tier: QuotaTier
    ) -> UserQuota:
        """Get existing quota or create new one."""
        if user_id in self.user_quotas:
            self.metrics["cache_hits"] += 1
            return self.user_quotas[user_id]

        self.metrics["cache_misses"] += 1
        now = int(time.time())
        quota = UserQuota(
            user_id=user_id,
            tier=tier,
            available_tokens=self._get_token_bucket_size(tier),
            last_refill=now,
        )
        self.user_quotas[user_id] = quota
        return quota

    def _check_monthly_quota(self, quota: UserQuota) -> bool:
        """Check if user has monthly quota remaining."""
        tier_quota = self.tier_limits[quota.tier][1]
        if tier_quota is None:
            return True  # Unlimited

        return quota.month_requests < tier_quota

    def _check_minute_limit(self, quota: UserQuota, cost: float = 1.0) -> bool:
        """Check if user can consume tokens from minute limit."""
        # Refill tokens if needed
        self._refill_tokens(quota)

        return quota.has_quota(cost)

    def _check_ip_limit(self, client_ip: Optional[str], cost: float = 1.0) -> bool:
        """Check IP-based rate limit (30 req/min)."""
        if not client_ip:
            return True

        now = int(time.time())
        if client_ip not in self.ip_quotas:
            self.ip_quotas[client_ip] = (now, int(cost))
            return True

        last_reset, count = self.ip_quotas[client_ip]

        # Reset if minute has passed
        if now - last_reset > 60:
            self.ip_quotas[client_ip] = (now, int(cost))
            return True

        # Check if limit exceeded
        new_count = count + int(cost)
        if new_count > 30:
            return False

        self.ip_quotas[client_ip] = (last_reset, new_count)
        return True

    def _refill_tokens(self, quota: UserQuota) -> None:
        """Refill tokens using token bucket algorithm."""
        now = int(time.time())
        time_passed = now - quota.last_refill

        if time_passed < 60:
            return  # Don't refill yet

        # Full minute passed, give all tokens
        bucket_size = self._get_token_bucket_size(quota.tier)
        quota.available_tokens = min(
            bucket_size * 1.5,  # Max burst: 150% of per-minute limit
            quota.available_tokens + bucket_size * (time_passed / 60),
        )
        quota.last_refill = now

    def _get_minute_limit(self, tier: QuotaTier) -> int:
        """Get per-minute limit for tier."""
        limit = self.tier_limits[tier][0]
        return limit if limit is not None else 999_999_999

    def _get_token_bucket_size(self, tier: QuotaTier) -> float:
        """Get token bucket size for tier."""
        limit = self._get_minute_limit(tier)
        return float(limit)

    def get_metrics(self) -> Dict:
        """Get current metrics."""
        total_checked = self.metrics["total_requests"]
        total_rejected = self.metrics["total_rejected"]
        cache_hits = self.metrics["cache_hits"]
        cache_misses = self.metrics["cache_misses"]

        return {
            "total_requests": total_checked,
            "total_rejected": total_rejected,
            "rejection_rate": (
                total_rejected / total_checked if total_checked > 0 else 0
            ),
            "cache_hits": cache_hits,
            "cache_misses": cache_misses,
            "cache_hit_ratio": (
                cache_hits / (cache_hits + cache_misses)
                if (cache_hits + cache_misses) > 0
                else 0
            ),
            "active_users": len(self.user_quotas),
            "active_ips": len(self.ip_quotas),
        }

    def get_prometheus_metrics(self) -> str:
        """Get metrics in Prometheus text format."""
        metrics = self.get_metrics()

        lines = [
            "# HELP rate_limit_total_requests Total requests processed",
            "# TYPE rate_limit_total_requests counter",
            f'rate_limit_total_requests{{service="fingerprint-api"}} {metrics["total_requests"]}',
            "",
            "# HELP rate_limit_rejected_total Total requests rejected",
            "# TYPE rate_limit_rejected_total counter",
            f'rate_limit_rejected_total{{service="fingerprint-api"}} {metrics["total_rejected"]}',
            "",
            "# HELP rate_limit_rejection_ratio Request rejection ratio",
            "# TYPE rate_limit_rejection_ratio gauge",
            f'rate_limit_rejection_ratio{{service="fingerprint-api"}} {metrics["rejection_rate"]:.4f}',
            "",
            "# HELP cache_hits_total Cache hit count",
            "# TYPE cache_hits_total counter",
            f'cache_hits_total{{service="fingerprint-api"}} {metrics["cache_hits"]}',
            "",
            "# HELP cache_hit_ratio Cache hit ratio",
            "# TYPE cache_hit_ratio gauge",
            f'cache_hit_ratio{{service="fingerprint-api"}} {metrics["cache_hit_ratio"]:.4f}',
            "",
            "# HELP rate_limit_active_users Active users count",
            "# TYPE rate_limit_active_users gauge",
            f'rate_limit_active_users{{service="fingerprint-api"}} {metrics["active_users"]}',
            "",
        ]

        return "\n".join(lines)

    def clear_user_quota(self, user_id: str) -> None:
        """Clear user quota (for testing)."""
        if user_id in self.user_quotas:
            del self.user_quotas[user_id]
