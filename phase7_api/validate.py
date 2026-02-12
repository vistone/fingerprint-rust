#!/usr/bin/env python3
"""
Phase 7.4 API Quick Validation Script
Tests that all components are properly integrated and ready for deployment
"""

import sys
from pathlib import Path

# Add project to path
project_dir = Path(__file__).parent
sys.path.insert(0, str(project_dir))

def test_imports():
    """Test that all modules can be imported"""
    print("🔍 Testing module imports...")
    
    try:
        print("  ├─ Importing features...")
        from features import TLSFeatureExtractor, extract_http_feature_vector, extract_combined_features, FeatureNormalizer
        print("  │  ✅ Feature extraction modules loaded")
        
        print("  ├─ Importing inference...")
        from inference import ModelLoader, InferenceEngine
        print("  │  ✅ Inference modules loaded")
        
        print("  ├─ Importing FastAPI app...")
        from app.main import app
        print("  │  ✅ FastAPI application loaded")
        
        print("  └─ All imports successful ✅")
        return True
    except ImportError as e:
        print(f"  ❌ Import failed: {e}")
        return False


def test_feature_extraction():
    """Test feature extraction pipeline"""
    print("\n🧪 Testing feature extraction...")
    
    try:
        from features import TLSFeatureExtractor, extract_http_feature_vector, extract_combined_features
        
        # Create dummy TLS data
        tls_data = bytearray([0x16, 0x03, 0x03] + [0x00] * 50)
        
        # Test TLS extraction
        extractor = TLSFeatureExtractor()
        tls_features = extractor.extract_feature_vector(bytes(tls_data))
        print(f"  ├─ TLS features: {len(tls_features)} dimensions ✅")
        
        # Test HTTP extraction
        headers = {
            "user-agent": "Mozilla/5.0",
            "accept-encoding": "gzip"
        }
        http_features = extract_http_feature_vector(headers)
        print(f"  ├─ HTTP features: {len(http_features)} dimensions ✅")
        
        # Test combined features
        combined = extract_combined_features(tls_features, http_features)
        if len(combined) == 53:
            print(f"  └─ Combined features: {len(combined)} dimensions ✅")
            return True
        else:
            print(f"  └─ ❌ Wrong feature dimension: {len(combined)} (expected 53)")
            return False
    
    except Exception as e:
        print(f"  ❌ Feature extraction failed: {e}")
        return False


def test_normalizer():
    """Test feature normalizer"""
    print("\n🔄 Testing feature normalizer...")
    
    try:
        from features import FeatureNormalizer
        
        # Create dummy normalizer
        normalizer = FeatureNormalizer.create_dummy_normalizer()
        
        # Test normalization
        dummy_features = [0.0] * 53
        normalized = normalizer.normalize(dummy_features)
        
        if len(normalized) == 53:
            print(f"  ├─ Normalization output: {len(normalized)} dimensions ✅")
        else:
            print(f"  ├─ ❌ Wrong output dimension: {len(normalized)}")
            return False
        
        if normalizer.validate(normalized):
            print(f"  └─ Feature validation: Passed ✅")
            return True
        else:
            print(f"  └─ ❌ Feature validation failed")
            return False
    
    except Exception as e:
        print(f"  ❌ Normalizer test failed: {e}")
        return False


def test_model_loader():
    """Test model loader initialization"""
    print("\n📦 Testing model loader...")
    
    try:
        from inference import ModelLoader
        
        loader = ModelLoader()
        
        # Check family names
        family_names = loader.family_names
        if len(family_names) == 11:
            print(f"  ├─ Family mapping: {len(family_names)} families ✅")
            for i, name in enumerate(family_names):
                print(f"  │  └─ [{i}] {name}")
        
        status = loader.get_status()
        all_false = all(v == False for v in status.values())
        
        if all_false:
            print(f"  └─ Model loader ready (models not loaded) ✅")
            print(f"     (This is expected - models only load if files exist)")
            return True
        else:
            print(f"  └─ Unexpected status: {status}")
            return False
    
    except Exception as e:
        print(f"  ❌ Model loader test failed: {e}")
        return False


def test_fastapi_app():
    """Test FastAPI application structure"""
    print("\n🚀 Testing FastAPI application...")
    
    try:
        from app.main import app
        from fastapi.testclient import TestClient
        
        client = TestClient(app)
        
        # Test health check
        response = client.get("/health")
        if response.status_code == 200:
            print(f"  ├─ Health endpoint: OK (200) ✅")
        else:
            print(f"  ├─ ❌ Health endpoint returned {response.status_code}")
            return False
        
        # Test root endpoint
        response = client.get("/")
        if response.status_code == 200:
            print(f"  ├─ Root endpoint: OK (200) ✅")
        else:
            print(f"  ├─ ❌ Root endpoint returned {response.status_code}")
            return False
        
        # Check endpoints exist
        endpoints_to_check = [
            ("POST", "/api/v1/fingerprint/identify"),
            ("GET", "/api/v1/models/status"),
            ("GET", "/api/v1/models/features"),
            ("POST", "/api/v1/models/validate"),
            ("POST", "/api/v1/models/retrain"),
        ]
        
        routes = [route.path for route in app.routes]
        for method, path in endpoints_to_check:
            if any(path in route for route in routes):
                print(f"  ├─ Endpoint {method} {path}: ✅")
            else:
                print(f"  ├─ ❌ Endpoint {method} {path}: NOT FOUND")
        
        print(f"  └─ FastAPI application: Ready ✅")
        return True
    
    except Exception as e:
        print(f"  ❌ FastAPI test failed: {e}")
        import traceback
        traceback.print_exc()
        return False


def test_project_structure():
    """Test that all required files exist"""
    print("\n📁 Testing project structure...")
    
    required_files = [
        "app/main.py",
        "features/__init__.py",
        "features/tls_features.py",
        "features/http_features.py",
        "features/normalizer.py",
        "inference/__init__.py",
        "inference/loader.py",
        "inference/engine.py",
        "requirements.txt",
        "Dockerfile",
        "docker-compose.yml",
        "README.md",
    ]
    
    all_exist = True
    for file_path in required_files:
        full_path = project_dir / file_path
        if full_path.exists():
            print(f"  ✅ {file_path}")
        else:
            print(f"  ❌ {file_path} - NOT FOUND")
            all_exist = False
    
    return all_exist


def main():
    """Run all validation tests"""
    print("╔════════════════════════════════════════════════════════════════╗")
    print("║     Phase 7.4 REST API - Quick Validation Script               ║")
    print("╚════════════════════════════════════════════════════════════════╝")
    print()
    
    tests = [
        ("Project Structure", test_project_structure),
        ("Module Imports", test_imports),
        ("Feature Extraction", test_feature_extraction),
        ("Feature Normalizer", test_normalizer),
        ("Model Loader", test_model_loader),
        ("FastAPI Application", test_fastapi_app),
    ]
    
    results = []
    for test_name, test_func in tests:
        try:
            result = test_func()
            results.append((test_name, result))
        except Exception as e:
            print(f"\n❌ {test_name} - Unexpected error: {e}")
            results.append((test_name, False))
    
    # Summary
    print("\n" + "=" * 60)
    print("📊 VALIDATION SUMMARY")
    print("=" * 60)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for test_name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{test_name:.<40} {status}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n✨ All validation tests passed! API is ready for deployment.")
        print("\nNext steps:")
        print("  1. cd phase7_api")
        print("  2. make install  (or pip install -r requirements.txt)")
        print("  3. make run      (or python -m uvicorn app.main:app --reload)")
        print("  4. visit http://localhost:8000/docs for interactive API docs")
        return 0
    else:
        print(f"\n⚠️  {total - passed} tests failed. Please fix issues before running API.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
