"""
Rate Limiting Tests

Unit and integration tests for rate limiting service and middleware.
"""

import pytest
import asyncio
from datetime import datetime
from unittest.mock import patch, MagicMock

from fingerprint_api.services.rate_limit_service import (
    RateLimitService,
    RateLimitError,
    QuotaTier,
    UserQuota,
)
from fingerprint_api.middleware.rate_limiter import RateLimitMiddleware
from fingerprint_api.config.rate_limit_config import RateLimitConfig


class TestQuotaTier:
    """Tests for QuotaTier enum."""

    def test_tier_values(self):
        """Test tier enum values."""
        assert QuotaTier.Free.value == "free"
        assert QuotaTier.Pro.value == "pro"
        assert QuotaTier.Enterprise.value == "enterprise"
        assert QuotaTier.Partner.value == "partner"


class TestUserQuota:
    """Tests for UserQuota class."""

    def test_user_quota_creation(self):
        """Test creating user quota."""
        now = int(__import__("time").time())
        quota = UserQuota(
            user_id="user123",
            tier=QuotaTier.Free,
            available_tokens=100.0,
            last_refill=now,
        )

        assert quota.user_id == "user123"
        assert quota.tier == QuotaTier.Free
        assert quota.available_tokens == 100.0
        assert quota.total_requests == 0

    def test_user_quota_consumption(self):
        """Test consuming tokens from quota."""
        now = int(__import__("time").time())
        quota = UserQuota(
            user_id="user123",
            tier=QuotaTier.Free,
            available_tokens=100.0,
            last_refill=now,
        )

        assert quota.has_quota(10.0)
        quota.consume(10.0)
        assert quota.available_tokens == 90.0
        assert quota.total_requests == 1

    def test_quota_exhaustion(self):
        """Test quota exhaustion."""
        now = int(__import__("time").time())
        quota = UserQuota(
            user_id="user123",
            tier=QuotaTier.Free,
            available_tokens=5.0,
            last_refill=now,
        )

        assert quota.has_quota(5.0)
        assert not quota.has_quota(6.0)


class TestRateLimitService:
    """Tests for RateLimitService class."""

    @pytest.fixture
    def service(self):
        """Create rate limit service for tests."""
        return RateLimitService(redis_enabled=False)

    def test_service_initialization(self, service):
        """Test service initialization."""
        assert service.redis_enabled is False
        assert service.metrics_enabled is True
        assert len(service.user_quotas) == 0

    def test_tier_limits(self, service):
        """Test tier definitions."""
        limits = service.tier_limits

        assert limits[QuotaTier.Free][0] == 100  # minute limit
        assert limits[QuotaTier.Free][1] == 50_000  # monthly quota

        assert limits[QuotaTier.Pro][0] == 1_000
        assert limits[QuotaTier.Pro][1] == 1_000_000

        assert limits[QuotaTier.Enterprise][0] is None  # unlimited
        assert limits[QuotaTier.Enterprise][1] is None

    def test_minute_limit_retrieval(self, service):
        """Test getting minute limit for tier."""
        assert service._get_minute_limit(QuotaTier.Free) == 100
        assert service._get_minute_limit(QuotaTier.Pro) == 1_000
        assert service._get_minute_limit(QuotaTier.Enterprise) == 999_999_999

    def test_check_limit_allowed(self, service):
        """Test checking limit when allowed."""
        response = service.check_limit(
            user_id="user123",
            user_tier=QuotaTier.Free,
            endpoint="/identify",
        )

        assert response["allowed"] is not None
        assert response["remaining"] >= 0
        assert response["reset_at"] > 0
        assert response["tier"].value == "free"

    def test_check_limit_rate_limited(self, service):
        """Test rate limit exceeded."""
        # Consume all tokens
        for _ in range(100):
            response = service.check_limit(
                user_id="user123",
                user_tier=QuotaTier.Free,
                endpoint="/identify",
            )

        # Next request should be rejected
        with pytest.raises(RateLimitError) as exc_info:
            service.check_limit(
                user_id="user123",
                user_tier=QuotaTier.Free,
                endpoint="/identify",
            )

        assert exc_info.value.error_type == "rate_limit_exceeded"

    def test_endpoint_cost_multiplier(self, service):
        """Test endpoint cost multipliers."""
        service.register_endpoint("/compare", cost=2.0)

        # Free tier: 100 req/min, so 50 with 2x cost
        for _ in range(50):
            service.check_limit(
                user_id="user123",
                user_tier=QuotaTier.Free,
                endpoint="/compare",
            )

        # 51st request should fail
        with pytest.raises(RateLimitError):
            service.check_limit(
                user_id="user123",
                user_tier=QuotaTier.Free,
                endpoint="/compare",
            )

    def test_ip_fallback_limit(self, service):
        """Test IP-based fallback when no auth."""
        # Consume IP limit (30 req/min default)
        for _ in range(30):
            response = service.check_limit(
                user_id=None,
                user_tier=QuotaTier.Free,
                endpoint="/identify",
                client_ip="192.168.1.1",
            )

        # Next request should be rejected
        with pytest.raises(RateLimitError):
            service.check_limit(
                user_id=None,
                user_tier=QuotaTier.Free,
                endpoint="/identify",
                client_ip="192.168.1.1",
            )

    def test_metrics_collection(self, service):
        """Test metrics collection."""
        # Generate some traffic
        for _ in range(10):
            service.check_limit(
                user_id="user123",
                user_tier=QuotaTier.Free,
                endpoint="/identify",
            )

        metrics = service.get_metrics()

        assert metrics["total_requests"] == 10
        assert metrics["total_rejected"] == 0
        assert metrics["cache_misses"] == 1
        assert metrics["cache_hits"] == 9

    def test_prometheus_format(self, service):
        """Test Prometheus metrics export."""
        service.check_limit(
            user_id="user123",
            user_tier=QuotaTier.Free,
            endpoint="/identify",
        )

        prometheus_text = service.get_prometheus_metrics()

        assert "rate_limit_total_requests" in prometheus_text
        assert "rate_limit_rejected_total" in prometheus_text
        assert "cache_hits_total" in prometheus_text
        assert "counter" in prometheus_text
        assert "gauge" in prometheus_text

    def test_token_bucket_refill(self, service):
        """Test token bucket refill logic."""
        quota = UserQuota(
            user_id="user123",
            tier=QuotaTier.Free,
            available_tokens=50.0,
            last_refill=int(__import__("time").time()) - 120,  # 2 minutes ago
        )

        service._refill_tokens(quota)

        # After 2 minutes, should have more tokens (with burst limit)
        assert quota.available_tokens > 50.0

    def test_clear_user_quota(self, service):
        """Test clearing user quota."""
        service.check_limit(
            user_id="user123",
            user_tier=QuotaTier.Free,
            endpoint="/identify",
        )

        assert "user123" in service.user_quotas

        service.clear_user_quota("user123")

        assert "user123" not in service.user_quotas


class TestMultipleUsers:
    """Tests for multiple concurrent users."""

    def test_multiple_user_quotas(self):
        """Test handling multiple users independently."""
        service = RateLimitService(redis_enabled=False)

        # Add requests from multiple users
        for user_id in ["user1", "user2", "user3"]:
            for _ in range(50):
                service.check_limit(
                    user_id=user_id,
                    user_tier=QuotaTier.Free,
                    endpoint="/identify",
                )

        # Each user should have their own quota
        for user_id in ["user1", "user2", "user3"]:
            quota = service.user_quotas[user_id]
            assert quota.total_requests == 50
            assert quota.available_tokens < 100  # Consumed

    def test_different_tier_limits(self):
        """Test different tiers have different limits."""
        service = RateLimitService(redis_enabled=False)

        # Free tier: 100 req/min
        free_count = 0
        while True:
            try:
                service.check_limit(
                    user_id="free_user",
                    user_tier=QuotaTier.Free,
                    endpoint="/identify",
                )
                free_count += 1
            except RateLimitError:
                break

        # Pro tier: 1000 req/min
        pro_count = 0
        while True:
            try:
                service.check_limit(
                    user_id="pro_user",
                    user_tier=QuotaTier.Pro,
                    endpoint="/identify",
                )
                pro_count += 1
            except RateLimitError:
                break

        assert free_count == 100
        assert pro_count == 1_000
        assert pro_count > free_count


class TestErrorScenarios:
    """Tests for error scenarios."""

    def test_rate_limit_error_attributes(self):
        """Test RateLimitError attributes."""
        error = RateLimitError(
            error_type="test_error",
            message="Test message",
            retry_after=120,
            reset_at=1234567890,
        )

        assert error.error_type == "test_error"
        assert error.message == "Test message"
        assert error.retry_after == 120
        assert error.reset_at == 1234567890

    def test_monthly_quota_exceeded(self):
        """Test monthly quota exhaustion."""
        service = RateLimitService(redis_enabled=False)

        # Free tier: 50k/month
        # Rapidly consume 50k requests
        quota = UserQuota(
            user_id="user123",
            tier=QuotaTier.Free,
            available_tokens=100.0,
            last_refill=int(__import__("time").time()),
            month_requests=50_000,
        )
        service.user_quotas["user123"] = quota

        # Manually adjust quota to avoid rate limit check
        quota.month_requests = 50_000

        # Next request should fail due to monthly quota
        with pytest.raises(RateLimitError) as exc_info:
            service.check_limit(
                user_id="user123",
                user_tier=QuotaTier.Free,
                endpoint="/identify",
            )

        assert exc_info.value.error_type == "monthly_quota_exceeded"


class TestConfiguration:
    """Tests for configuration."""

    def test_config_defaults(self):
        """Test configuration defaults."""
        config = RateLimitConfig()

        assert config.ENABLED is True
        assert config.REDIS_ENABLED is True
        assert config.METRICS_ENABLED is True
        assert config.FREE_TIER_MINUTE_LIMIT == 100
        assert config.FREE_TIER_MONTHLY_QUOTA == 50_000


if __name__ == "__main__":
    # Run tests with: pytest tests/test_rate_limiting.py -v
    pytest.main([__file__, "-v", "--tb=short"])
