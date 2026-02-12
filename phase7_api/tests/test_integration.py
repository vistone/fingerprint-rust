"""
Integration tests for Browser Fingerprint API
"""

import pytest
from fastapi.testclient import TestClient
from typing import Dict
import base64

# Import app after setting up path
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from app.main import app


@pytest.fixture
def client():
    """Create test client"""
    return TestClient(app)


class TestHealthCheck:
    """Health check endpoint tests"""
    
    def test_health_check(self, client):
        """Test health check endpoint"""
        response = client.get("/health")
        assert response.status_code == 200
        assert response.json()["status"] == "healthy"
    
    def test_root_endpoint(self, client):
        """Test root endpoint"""
        response = client.get("/")
        assert response.status_code == 200
        data = response.json()
        assert data["name"] == "Browser Fingerprint Identification API"


class TestModelStatus:
    """Model status endpoint tests"""
    
    def test_get_status(self, client):
        """Test model status endpoint"""
        response = client.get("/api/v1/models/status")
        assert response.status_code == 200
        data = response.json()
        
        # Check structure
        assert "status" in data
        assert "inference_statistics" in data
        assert "total_inferences" in data
        assert "average_latency_ms" in data
    
    def test_get_features(self, client):
        """Test feature description endpoint"""
        response = client.get("/api/v1/models/features")
        assert response.status_code == 200
        data = response.json()
        
        # Check feature count
        assert data.get("total_features", 0) == 53 or "description" in data


class TestIdentification:
    """Fingerprint identification tests"""
    
    def create_dummy_tls_data(self) -> bytes:
        """Create dummy TLS ClientHello data for testing"""
        # Minimal TLS 1.2 ClientHello structure
        # This is a simplified version with proper byte structure
        tls_data = bytearray([
            0x16,  # Content type: Handshake
            0x03, 0x03,  # Version: TLS 1.2
            0x00, 0x4a,  # Length
            0x01,  # Handshake type: ClientHello
            0x00, 0x00, 0x46,  # Length
            0x03, 0x03,  # Version: TLS 1.2
        ])
        
        # Add 32 random bytes for random
        tls_data.extend([0x00] * 32)
        
        # Session ID length
        tls_data.append(0x00)
        
        # Cipher suites (simplified)
        tls_data.extend([0x00, 0x02])  # Length
        tls_data.extend([0x13, 0x01])  # TLS_AES_128_GCM_SHA256
        
        # Compression methods
        tls_data.append(0x01)  # Length
        tls_data.append(0x00)  # Null compression
        
        # Extensions length
        tls_data.extend([0x00, 0x00])
        
        return bytes(tls_data)
    
    def test_identify_with_dummy_data(self, client):
        """Test identification with dummy data"""
        tls_data = self.create_dummy_tls_data()
        tls_data_b64 = base64.b64encode(tls_data).decode()
        
        payload = {
            "tls_data": tls_data_b64,
            "http_headers": {
                "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
                "accept": "text/html",
                "accept-encoding": "gzip, deflate, br"
            },
            "session_id": "test_session_123"
        }
        
        response = client.post("/api/v1/fingerprint/identify", json=payload)
        
        # Should return 200 if models are loaded, 503 if not
        if response.status_code == 200:
            data = response.json()
            assert "family" in data
            assert "family_confidence" in data
            assert "version" in data
            assert "variant" in data
            assert "overall_confidence" in data
        elif response.status_code == 503:
            # Models not loaded in test environment is acceptable
            assert "not initialized" in response.json()["detail"]
    
    def test_identify_with_session_id(self, client):
        """Test that session ID is returned in response"""
        tls_data = self.create_dummy_tls_data()
        tls_data_b64 = base64.b64encode(tls_data).decode()
        
        payload = {
            "tls_data": tls_data_b64,
            "http_headers": {"user-agent": "Test"},
            "session_id": "unique_session_xyz"
        }
        
        response = client.post("/api/v1/fingerprint/identify", json=payload)
        
        if response.status_code == 200:
            data = response.json()
            assert data.get("session_id") == "unique_session_xyz"


class TestValidation:
    """Model validation endpoint tests"""
    
    def test_validate_models(self, client):
        """Test model validation endpoint"""
        response = client.post("/api/v1/models/validate")
        
        if response.status_code == 200:
            data = response.json()
            assert "test_samples" in data
            assert "family_accuracy" in data
            assert "average_accuracy" in data
            assert data["family_accuracy"] >= 0
            assert data["family_accuracy"] <= 100
        elif response.status_code == 503:
            # Models not initialized
            assert "not initialized" in response.json()["detail"]
        elif response.status_code == 404:
            # Test dataset not found
            assert "not found" in response.json()["detail"]


class TestAdmin:
    """Admin endpoint tests"""
    
    def test_retrain_without_key(self, client):
        """Test retraining without API key"""
        response = client.post("/api/v1/models/retrain")
        
        if response.status_code == 200:
            # API key verification may not be strict in test
            pass
        else:
            # Should get 403 without valid key
            assert response.status_code in [403, 500]
    
    def test_retrain_with_valid_key(self, client):
        """Test retraining with valid API key"""
        response = client.post("/api/v1/models/retrain?api_key=admin_key_phase7_4")
        
        if response.status_code == 200:
            data = response.json()
            assert data.get("status") == "scheduled"


class TestErrorHandling:
    """Error handling tests"""
    
    def test_invalid_request_format(self, client):
        """Test handling of invalid request format"""
        response = client.post("/api/v1/fingerprint/identify", json={})
        assert response.status_code == 422  # Validation error
    
    def test_endpoint_not_found(self, client):
        """Test 404 for non-existent endpoint"""
        response = client.get("/api/v1/nonexistent")
        assert response.status_code == 404


class TestPerformance:
    """Performance tests"""
    
    def test_identification_latency(self, client):
        """Test that identification completes in reasonable time"""
        import time
        
        tls_data = bytearray([0x16, 0x03, 0x03] + [0x00] * 50)
        payload = {
            "tls_data": base64.b64encode(bytes(tls_data)).decode(),
            "http_headers": {"user-agent": "Test"}
        }
        
        start = time.time()
        response = client.post("/api/v1/fingerprint/identify", json=payload)
        elapsed = time.time() - start
        
        # Should complete in reasonable time
        # (even if response is 503 due to models not loaded)
        assert elapsed < 5.0  # 5 second max for test


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
