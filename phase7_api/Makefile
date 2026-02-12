.PHONY: help install run test docker-build docker-up docker-down clean lint format

help:
	@echo "Browser Fingerprint API - Phase 7.4"
	@echo ""
	@echo "Available targets:"
	@echo "  install         Install Python dependencies"
	@echo "  run            Start API server (development)"
	@echo "  run-prod       Start API server (production with uvicorn)"
	@echo "  test           Run all tests"
	@echo "  test-unit      Run unit tests only"
	@echo "  test-integration Run integration tests"
	@echo "  test-perf      Run performance benchmarks"
	@echo "  docker-build   Build Docker image"
	@echo "  docker-up      Start Docker container (compose)"
	@echo "  docker-down    Stop Docker container (compose)"
	@echo "  clean          Remove temporary files"
	@echo "  lint           Check code style (flake8)"
	@echo "  format         Format code (black)"
	@echo "  docs           Open interactive API docs"

install:
	@echo "Installing dependencies..."
	python3 -m pip install -r requirements.txt
	@echo "✅ Dependencies installed"

run:
	@echo "Starting API (development mode)..."
	python3 -m uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

run-prod:
	@echo "Starting API (production mode)..."
	python3 -m uvicorn app.main:app --host 0.0.0.0 --port 8000 --workers 2

test:
	@echo "Running all tests..."
	python3 -m pytest tests/ -v

test-unit:
	@echo "Running unit tests..."
	python3 -m pytest tests/ -v -m "not slow"

test-integration:
	@echo "Running integration tests..."
	python3 -m pytest tests/test_integration.py -v

test-perf:
	@echo "Running performance benchmarks..."
	python3 -m pytest tests/test_performance.py -v -s -m slow

docker-build:
	@echo "Building Docker image..."
	docker build -t fingerprint-api:1.0 .
	@echo "✅ Docker image built: fingerprint-api:1.0"

docker-up:
	@echo "Starting Docker container..."
	docker-compose up -d
	@echo "✅ API running at http://localhost:8000"
	@echo "📚 Docs at http://localhost:8000/docs"

docker-down:
	@echo "Stopping Docker container..."
	docker-compose down
	@echo "✅ Container stopped"

docker-logs:
	docker-compose logs -f fingerprint-api

clean:
	@echo "Cleaning up..."
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type d -name .pytest_cache -exec rm -rf {} +
	find . -type d -name .mypy_cache -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	@echo "✅ Cleanup complete"

lint:
	@echo "Checking code style..."
	python3 -m flake8 app features inference --max-line-length=100 || true

format:
	@echo "Formatting code..."
	python3 -m black app features inference tests --line-length=100

docs:
	@echo "Opening API documentation..."
	@if command -v xdg-open > /dev/null; then \
		xdg-open http://localhost:8000/docs; \
	elif command -v open > /dev/null; then \
		open http://localhost:8000/docs; \
	else \
		echo "Please open http://localhost:8000/docs in your browser"; \
	fi

status:
	@echo "API Status:"
	@curl -s http://localhost:8000/health | python3 -m json.tool || echo "API not running"

models-status:
	@echo "Model Status:"
	@curl -s http://localhost:8000/api/v1/models/status | python3 -m json.tool || echo "API not running"

quick-test:
	@echo "Running quick integration test..."
	python3 -m pytest tests/test_integration.py::TestHealthCheck::test_health_check -v

.DEFAULT_GOAL := help
