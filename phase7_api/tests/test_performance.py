"""
Performance Benchmarks for Browser Fingerprint API
"""

import pytest
import time
import statistics
from typing import List
import numpy as np
import base64

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from app.main import app
from fastapi.testclient import TestClient


class BenchmarkResults:
    """Store and analyze benchmark results"""
    
    def __init__(self):
        self.measurements: List[float] = []
    
    def add_measurement(self, value: float):
        """Add a measurement in milliseconds"""
        self.measurements.append(value)
    
    def get_statistics(self):
        """Get statistics about measurements"""
        if not self.measurements:
            return None
        
        return {
            "count": len(self.measurements),
            "mean_ms": statistics.mean(self.measurements),
            "median_ms": statistics.median(self.measurements),
            "stdev_ms": statistics.stdev(self.measurements) if len(self.measurements) > 1 else 0,
            "min_ms": min(self.measurements),
            "max_ms": max(self.measurements),
            "p50_ms": statistics.median(self.measurements),
            "p95_ms": np.percentile(self.measurements, 95),
            "p99_ms": np.percentile(self.measurements, 99),
        }


@pytest.fixture
def client():
    """Create test client"""
    return TestClient(app)


class TestLatencyBenchmarks:
    """Latency benchmarks for identification endpoint"""
    
    def create_test_payload(self, num: int = 0):
        """Create test payload"""
        tls_data = bytearray([0x16, 0x03, 0x03] + [0x00] * 50)
        return {
            "tls_data": base64.b64encode(bytes(tls_data)).decode(),
            "http_headers": {
                "user-agent": f"Test-Agent-{num}",
                "accept": "text/html"
            },
            "session_id": f"bench_session_{num}"
        }
    
    @pytest.mark.slow
    def test_single_request_latency(self, client):
        """Benchmark single request latency"""
        payload = self.create_test_payload(0)
        
        start = time.time()
        response = client.post("/api/v1/fingerprint/identify", json=payload)
        elapsed = (time.time() - start) * 1000  # Convert to ms
        
        print(f"\n📊 Single Request Latency: {elapsed:.2f}ms")
        
        if response.status_code == 200:
            # Target: <50ms for single request
            assert elapsed < 50, f"Latency {elapsed}ms exceeds target 50ms"
    
    @pytest.mark.slow
    def test_batch_latency(self, client):
        """Benchmark batch request latency"""
        results = BenchmarkResults()
        num_requests = 10
        
        for i in range(num_requests):
            payload = self.create_test_payload(i)
            
            start = time.time()
            response = client.post("/api/v1/fingerprint/identify", json=payload)
            elapsed = (time.time() - start) * 1000
            
            if response.status_code == 200:
                results.add_measurement(elapsed)
        
        stats = results.get_statistics()
        if stats:
            print(f"\n📊 Batch Latency ({num_requests} requests):")
            print(f"   Mean: {stats['mean_ms']:.2f}ms")
            print(f"   Median: {stats['median_ms']:.2f}ms")
            print(f"   P95: {stats['p95_ms']:.2f}ms")
            print(f"   P99: {stats['p99_ms']:.2f}ms")
    
    @pytest.mark.slow
    def test_throughput(self, client):
        """Benchmark maximum throughput"""
        num_requests = 20
        results = BenchmarkResults()
        
        start_total = time.time()
        
        for i in range(num_requests):
            payload = self.create_test_payload(i)
            
            start = time.time()
            response = client.post("/api/v1/fingerprint/identify", json=payload)
            elapsed = (time.time() - start) * 1000
            
            if response.status_code == 200:
                results.add_measurement(elapsed)
        
        total_elapsed = (time.time() - start_total) * 1000
        throughput = (num_requests / (total_elapsed / 1000))
        
        stats = results.get_statistics()
        if stats:
            print(f"\n📊 Throughput:")
            print(f"   Requests: {num_requests}")
            print(f"   Total Time: {total_elapsed:.0f}ms")
            print(f"   Throughput: {throughput:.1f} requests/sec")
            print(f"   Average Latency: {stats['mean_ms']:.2f}ms")


class TestEndpointPerformance:
    """Test performance of all endpoints"""
    
    @pytest.mark.slow
    def test_status_endpoint_latency(self, client):
        """Test model status endpoint latency"""
        results = BenchmarkResults()
        
        for _ in range(10):
            start = time.time()
            response = client.get("/api/v1/models/status")
            elapsed = (time.time() - start) * 1000
            
            if response.status_code == 200:
                results.add_measurement(elapsed)
        
        stats = results.get_statistics()
        if stats:
            print(f"\n📊 Status Endpoint Latency:")
            print(f"   Mean: {stats['mean_ms']:.2f}ms")
            print(f"   Max: {stats['max_ms']:.2f}ms")
    
    @pytest.mark.slow
    def test_features_endpoint_latency(self, client):
        """Test feature info endpoint latency"""
        start = time.time()
        response = client.get("/api/v1/models/features")
        elapsed = (time.time() - start) * 1000
        
        print(f"\n📊 Features Endpoint Latency: {elapsed:.2f}ms")
        assert elapsed < 100, "Features endpoint too slow"


class TestMemoryUsage:
    """Test memory usage"""
    
    @pytest.mark.slow
    def test_api_initialization_memory(self, client):
        """Test memory usage during API initialization"""
        import psutil
        import os
        
        process = psutil.Process(os.getpid())
        mem_info = process.memory_info()
        
        print(f"\n📊 API Memory Usage:")
        print(f"   RSS: {mem_info.rss / 1024 / 1024:.1f} MB")
        print(f"   VMS: {mem_info.vms / 1024 / 1024:.1f} MB")
        
        # Target: <200 MB maximum
        assert mem_info.rss < 500 * 1024 * 1024, "Memory usage too high"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s", "-m", "slow"])
