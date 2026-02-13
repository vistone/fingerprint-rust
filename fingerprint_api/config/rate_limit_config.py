"""
Rate Limiting Configuration

Configuration for rate limiting service and middleware.
Can be overridden via environment variables.
"""

import os
from typing import Optional, List, Dict
from pydantic_settings import BaseSettings


class RateLimitConfig(BaseSettings):
    """Rate limiting configuration."""

    # Service Configuration
    ENABLED: bool = True
    SERVICE_NAME: str = "fingerprint-api"
    ENVIRONMENT: str = os.getenv("ENVIRONMENT", "development")

    # Redis Configuration
    REDIS_ENABLED: bool = os.getenv("RATE_LIMIT_REDIS_ENABLED", "true").lower() == "true"
    REDIS_URL: str = os.getenv(
        "RATE_LIMIT_REDIS_URL",
        "redis://localhost:6379/0",
    )
    REDIS_POOL_SIZE: int = int(os.getenv("RATE_LIMIT_REDIS_POOL_SIZE", "10"))
    REDIS_TIMEOUT_SECONDS: int = int(
        os.getenv("RATE_LIMIT_REDIS_TIMEOUT_SECONDS", "5")
    )
    REDIS_COMMAND_TIMEOUT_SECONDS: int = int(
        os.getenv("RATE_LIMIT_REDIS_COMMAND_TIMEOUT_SECONDS", "2")
    )

    # Metrics Configuration
    METRICS_ENABLED: bool = os.getenv("RATE_LIMIT_METRICS_ENABLED", "true").lower() == "true"
    METRICS_EXPORT_INTERVAL_SECONDS: int = int(
        os.getenv("RATE_LIMIT_METRICS_EXPORT_INTERVAL_SECONDS", "30")
    )

    # Tier Limits (req/min, monthly quota)
    FREE_TIER_MINUTE_LIMIT: int = int(
        os.getenv("RATE_LIMIT_FREE_MINUTE_LIMIT", "100")
    )
    FREE_TIER_MONTHLY_QUOTA: int = int(
        os.getenv("RATE_LIMIT_FREE_MONTHLY_QUOTA", "50000")
    )

    PRO_TIER_MINUTE_LIMIT: int = int(
        os.getenv("RATE_LIMIT_PRO_MINUTE_LIMIT", "1000")
    )
    PRO_TIER_MONTHLY_QUOTA: int = int(
        os.getenv("RATE_LIMIT_PRO_MONTHLY_QUOTA", "1000000")
    )

    ENTERPRISE_TIER_MINUTE_LIMIT: Optional[int] = None  # Unlimited
    ENTERPRISE_TIER_MONTHLY_QUOTA: Optional[int] = None  # Unlimited

    PARTNER_TIER_MINUTE_LIMIT: Optional[int] = None  # Unlimited
    PARTNER_TIER_MONTHLY_QUOTA: Optional[int] = None  # Unlimited

    # IP-Based Rate Limiting
    IP_FALLBACK_LIMIT: int = int(
        os.getenv("RATE_LIMIT_IP_FALLBACK_LIMIT", "30")
    )
    IP_WHITELIST: List[str] = [
        ip.strip()
        for ip in os.getenv("RATE_LIMIT_IP_WHITELIST", "10.0.0.0/8").split(",")
    ]

    # Token Bucket Configuration
    BURST_MULTIPLIER: float = float(
        os.getenv("RATE_LIMIT_BURST_MULTIPLIER", "1.5")
    )
    TOKEN_REFILL_INTERVAL_SECONDS: int = int(
        os.getenv("RATE_LIMIT_TOKEN_REFILL_INTERVAL_SECONDS", "60")
    )

    # Middleware Configuration
    MIDDLEWARE_ENABLED: bool = os.getenv("RATE_LIMIT_MIDDLEWARE_ENABLED", "true").lower() == "true"
    
    # Exempt paths (exclude from rate limiting)
    EXEMPT_PATHS: List[str] = [
        "/health",
        "/metrics",
        "/docs",
        "/redoc",
        "/openapi.json",
    ]

    # Endpoint Costs
    ENDPOINT_COSTS: Dict[str, float] = {
        "/identify": 1.0,
        "/compare": 2.0,
        "/batch": 1.0,
        "/health": 0.0,
        "/metrics": 0.0,
    }

    # Cache Configuration
    CACHE_MAX_SIZE: int = int(
        os.getenv("RATE_LIMIT_CACHE_MAX_SIZE", "10000")
    )
    CACHE_TTL_SECONDS: int = int(
        os.getenv("RATE_LIMIT_CACHE_TTL_SECONDS", "3600")
    )
    CACHE_CLEANUP_INTERVAL_SECONDS: int = int(
        os.getenv("RATE_LIMIT_CACHE_CLEANUP_INTERVAL_SECONDS", "300")
    )

    # Response Configuration
    INCLUDE_RATE_LIMIT_HEADERS: bool = True
    INCLUDE_RETRY_AFTER_HEADER: bool = True
    RESPONSE_TIME_HEADER: str = "X-Response-Time"

    # Logging Configuration
    LOG_LEVEL: str = os.getenv("RATE_LIMIT_LOG_LEVEL", "INFO")
    LOG_REJECTED_REQUESTS: bool = True
    LOG_METRICS_INTERVAL_SECONDS: int = int(
        os.getenv("RATE_LIMIT_LOG_METRICS_INTERVAL_SECONDS", "60")
    )

    # Alert Thresholds
    ALERT_REJECTION_RATE_THRESHOLD: float = float(
        os.getenv("RATE_LIMIT_ALERT_REJECTION_RATE_THRESHOLD", "0.05")
    )  # 5%
    ALERT_ERROR_RATE_THRESHOLD: float = float(
        os.getenv("RATE_LIMIT_ALERT_ERROR_RATE_THRESHOLD", "0.01")
    )  # 1%

    # Auth Configuration
    REQUIRE_AUTHENTICATION: bool = os.getenv(
        "RATE_LIMIT_REQUIRE_AUTHENTICATION", "true"
    ).lower() == "true"
    AUTH_HEADER_NAME: str = os.getenv("RATE_LIMIT_AUTH_HEADER_NAME", "X-API-Key")
    FALLBACK_AUTH_METHODS: List[str] = ["X-API-Key", "Authorization", "api_key"]

    class Config:
        """Pydantic config."""

        env_prefix = "RATE_LIMIT_"
        case_sensitive = True
        env_file = ".env.rate_limit"
        env_file_encoding = "utf-8"


def get_config() -> RateLimitConfig:
    """Get rate limiting configuration."""
    return RateLimitConfig()


# Default configuration instance
config = get_config()


# Configuration templates for different environments
DEVELOPMENT_CONFIG = RateLimitConfig(
    ENVIRONMENT="development",
    REDIS_ENABLED=False,  # Use in-memory only
    LOG_LEVEL="DEBUG",
    METRICS_ENABLED=True,
)

STAGING_CONFIG = RateLimitConfig(
    ENVIRONMENT="staging",
    REDIS_ENABLED=True,
    LOG_LEVEL="INFO",
    METRICS_ENABLED=True,
    MIDDLEWARE_ENABLED=True,
)

PRODUCTION_CONFIG = RateLimitConfig(
    ENVIRONMENT="production",
    REDIS_ENABLED=True,
    LOG_LEVEL="WARNING",
    METRICS_ENABLED=True,
    MIDDLEWARE_ENABLED=True,
    CACHE_MAX_SIZE=100000,
    CACHE_TTL_SECONDS=7200,
)


def get_environment_config() -> RateLimitConfig:
    """Get configuration based on environment."""
    env = os.getenv("ENVIRONMENT", "development").lower()

    if env == "production":
        return PRODUCTION_CONFIG
    elif env == "staging":
        return STAGING_CONFIG
    else:
        return DEVELOPMENT_CONFIG
