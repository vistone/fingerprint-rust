"""
FastAPI Integration Example

Complete example of integrating rate limiting middleware with FastAPI.
"""

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from contextlib import asynccontextmanager
import logging

# Rate limiting imports
from fingerprint_api.middleware.rate_limiter import (
    setup_rate_limiting,
    RateLimitMiddleware,
)
from fingerprint_api.routes.rate_limit_routes import setup_rate_limit_routes
from fingerprint_api.services.rate_limit_service import (
    RateLimitService,
    RateLimitError,
    QuotaTier,
)
from fingerprint_api.config.rate_limit_config import get_environment_config

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


# Create rate limit service instance
rate_limit_service: RateLimitService = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Application lifecycle management.
    Initialize and shutdown resources.
    """
    global rate_limit_service

    # Startup
    logger.info("Initializing rate limiting service...")
    config = get_environment_config()
    rate_limit_service = RateLimitService(
        redis_url=config.REDIS_URL,
        redis_enabled=config.REDIS_ENABLED,
        metrics_enabled=config.METRICS_ENABLED,
    )

    # Register endpoints with their costs
    for endpoint, cost in config.ENDPOINT_COSTS.items():
        rate_limit_service.register_endpoint(endpoint, cost)

    # Initialize async resources
    if config.REDIS_ENABLED:
        await rate_limit_service.initialize()

    logger.info(
        "Rate limiting service initialized. "
        f"Redis: {rate_limit_service.redis_enabled}, "
        f"Metrics: {rate_limit_service.metrics_enabled}"
    )

    yield

    # Shutdown
    logger.info("Shutting down rate limiting service...")
    await rate_limit_service.shutdown()


# Initialize FastAPI app
app = FastAPI(
    title="Fingerprint API with Rate Limiting",
    description="Fingerprint identification API with Phase 9.4 rate limiting",
    version="1.0.0",
    lifespan=lifespan,
)


# Setup middleware BEFORE routes (middleware runs in reverse order)
# Middleware order: CORS → Metrics → Rate Limiting
setup_rate_limiting(app, rate_limit_service)

# Setup rate limit management routes
setup_rate_limit_routes(app)


# Example endpoints
@app.get(
    "/identify",
    tags=["Fingerprinting"],
    summary="Identify User",
    description="Identify a user based on fingerprint",
)
async def identify(request: Request, headers: dict = None):
    """
    Identify endpoint (cost: 1.0 token).
    
    Rate limit headers in response:
    - X-RateLimit-Remaining: Tokens remaining in minute
    - X-RateLimit-Reset: Unix timestamp when limit resets
    - X-Quota-Tier: User's tier (free, pro, enterprise, partner)
    - X-Quota-Monthly-Remaining: Monthly quota remaining
    - Retry-After: (if rejected) Seconds to wait
    """
    return {
        "status": "identified",
        "fingerprint": {
            "ja4": "t13d11,11111,00000000",
            "tls_version": "TLS 1.3",
            "cipher_suites": 11,
        },
    }


@app.post(
    "/compare",
    tags=["Fingerprinting"],
    summary="Compare Fingerprints",
    description="Compare two fingerprints (cost: 2.0 tokens)",
)
async def compare(request: Request):
    """
    Compare endpoint (cost: 2.0 tokens).
    Consumes tokens at 2x rate.
    """
    body = await request.json()

    return {
        "status": "compared",
        "similarity": 0.95,
        "match": True,
    }


@app.post(
    "/batch",
    tags=["Fingerprinting"],
    summary="Batch Process",
    description="Batch process multiple fingerprints (cost: 1.0 token)",
)
async def batch(request: Request):
    """
    Batch endpoint (cost: 1.0 token).
    For processing multiple fingerprints efficiently.
    """
    body = await request.json()
    fingerprints = body.get("fingerprints", [])

    return {
        "status": "processed",
        "count": len(fingerprints),
        "results": [
            {"fingerprint": fp, "identified": True} for fp in fingerprints
        ],
    }


@app.get("/health", tags=["Health"])
async def health():
    """Health check endpoint (exempt from rate limiting)."""
    return {"status": "healthy", "service": "fingerprint-api"}


@app.get("/docs", include_in_schema=False)
async def docs():
    """OpenAPI documentation (exempt from rate limiting)."""
    return {"message": "See /docs for Swagger UI"}


# Error handlers
@app.exception_handler(RateLimitError)
async def rate_limit_exception_handler(request: Request, exc: RateLimitError):
    """Handle rate limit exceptions."""
    import time

    now = int(time.time())

    return JSONResponse(
        status_code=429,
        content={
            "error": exc.error_type,
            "message": exc.message,
            "retry_after": exc.retry_after,
            "reset_at": exc.reset_at or (now + 60),
        },
        headers={
            "Retry-After": str(exc.retry_after or 60),
            "X-RateLimit-Reset": str(exc.reset_at or now + 60),
        },
    )


@app.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception):
    """Handle general exceptions."""
    logger.error(f"Unhandled exception: {exc}", exc_info=True)

    return JSONResponse(
        status_code=500,
        content={
            "error": "internal_server_error",
            "message": "An internal server error occurred",
        },
    )


if __name__ == "__main__":
    import uvicorn

    # Run with: python -m uvicorn examples.phase_9_4_fastapi_integration:app --reload

    logger.info("Starting Fingerprint API with Rate Limiting...")
    logger.info("Available endpoints:")
    logger.info("  POST   /identify        - Single fingerprint identification")
    logger.info("  POST   /compare         - Compare two fingerprints")
    logger.info("  POST   /batch           - Batch process fingerprints")
    logger.info("  GET    /health          - Health check")
    logger.info("  GET    /docs            - API documentation (Swagger UI)")
    logger.info("")
    logger.info("Rate Limit Management endpoints:")
    logger.info("  GET    /api/v1/rate-limit/status       - Service status")
    logger.info("  GET    /api/v1/rate-limit/health       - Health check")
    logger.info("  GET    /api/v1/rate-limit/metrics      - Prometheus metrics")
    logger.info("  GET    /api/v1/rate-limit/metrics/json - JSON metrics")
    logger.info("  GET    /api/v1/rate-limit/tiers        - Tier definitions")
    logger.info("  GET    /api/v1/rate-limit/quota/{user_id} - User quota info")
    logger.info("")
    logger.info("Example requests:")
    logger.info("")
    logger.info("Free tier (default):")
    logger.info("  curl -X POST http://localhost:8000/identify")
    logger.info("")
    logger.info("Pro tier:")
    logger.info("  curl -H 'X-Quota-Tier: pro' http://localhost:8000/identify")
    logger.info("")
    logger.info("With API key:")
    logger.info("  curl -H 'X-API-Key: user123' http://localhost:8000/identify")
    logger.info("")

    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_level="info",
    )
