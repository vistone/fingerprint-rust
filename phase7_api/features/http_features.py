"""
HTTP Feature Extraction for Browser Fingerprinting
Extracts features from HTTP headers and requests
"""

from typing import List, Dict, Any


def extract_http_features(headers: Dict[str, str]) -> Dict[str, Any]:
    """
    Extract HTTP features from request headers
    
    Extracts 6 features:
    1. header_count
    2. Accept_Encoding presence
    3. Accept_Language presence
    4. User_Agent presence
    5. Content_Type presence
    6. Custom headers count
    """
    
    features = {
        "header_count": len(headers),
        "has_accept_encoding": 1 if "accept-encoding" in headers else 0,
        "has_accept_language": 1 if "accept-language" in headers else 0,
        "has_user_agent": 1 if "user-agent" in headers else 0,
        "has_content_type": 1 if "content-type" in headers else 0,
        "custom_headers_count": len([k for k in headers.keys() if k.startswith("x-")]),
    }
    
    return features


def extract_http_feature_vector(headers: Dict[str, str]) -> List[float]:
    """
    Extract HTTP features as a vector for ML models
    Returns 6-element feature vector
    """
    features = extract_http_features(headers)
    
    vector = [
        float(features.get("header_count", 0)),
        float(features.get("has_accept_encoding", 0)),
        float(features.get("has_accept_language", 0)),
        float(features.get("has_user_agent", 0)),
        float(features.get("has_content_type", 0)),
        float(features.get("custom_headers_count", 0)),
    ]
    
    return vector


def extract_combined_features(tls_vector: List[float], http_vector: List[float]) -> List[float]:
    """
    Combine TLS and HTTP feature vectors
    TLS: 12 features
    HTTP: 6 features
    Additional: 35 features (for total of 53)
    
    Returns 53-element feature vector
    """
    combined = tls_vector + http_vector
    
    # Add placeholder features to reach 53 dimensions
    # In production, these would be additional TLS/HTTP features
    additional_features = [0.0] * (53 - len(combined))
    combined.extend(additional_features)
    
    return combined[:53]  # Ensure exactly 53 features
