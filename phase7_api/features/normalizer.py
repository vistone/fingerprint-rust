"""
Feature Normalization for Browser Fingerprint ML Model
Standardizes 53-dimensional feature vectors
"""

from typing import List, Optional
import pickle
import numpy as np
from pathlib import Path


class FeatureNormalizer:
    """Normalize feature vectors using saved scaler from training"""
    
    def __init__(self, scaler_path: Optional[str] = None):
        """
        Initialize with optional scaler from training phase
        
        Args:
            scaler_path: Path to pickled StandardScaler
        """
        self.scaler = None
        self.scaler_path = scaler_path
        self.feature_dim = 53  # Expected feature dimension
        
        if scaler_path:
            self.load_scaler(scaler_path)
    
    def load_scaler(self, scaler_path: str) -> bool:
        """
        Load pre-trained StandardScaler from pickle
        
        Args:
            scaler_path: Path to scaler.pkl from training phase
            
        Returns:
            True if loaded successfully, False otherwise
        """
        try:
            path = Path(scaler_path)
            if not path.exists():
                print(f"Warning: Scaler file not found at {scaler_path}")
                return False
            
            with open(path, "rb") as f:
                self.scaler = pickle.load(f)
            
            print(f"✓ Scaler loaded from {scaler_path}")
            return True
        except Exception as e:
            print(f"Error loading scaler: {e}")
            return False
    
    def normalize(self, features: List[float]) -> List[float]:
        """
        Normalize feature vector using loaded scaler
        
        Args:
            features: 53-dimensional feature vector
            
        Returns:
            Normalized feature vector (same dimension)
        """
        if len(features) != self.feature_dim:
            raise ValueError(f"Expected {self.feature_dim} features, got {len(features)}")
        
        if self.scaler is None:
            # If no scaler loaded, return features as-is with warning
            print("Warning: No scaler loaded, returning raw features")
            return features
        
        # Convert to numpy array and reshape for sklearn
        features_array = np.array(features).reshape(1, -1)
        
        # Apply normalization
        normalized = self.scaler.transform(features_array)[0]
        
        return normalized.tolist()
    
    def validate(self, features: List[float]) -> bool:
        """
        Validate feature vector format and values
        
        Args:
            features: Feature vector to validate
            
        Returns:
            True if valid, False otherwise
        """
        # Check dimension
        if len(features) != self.feature_dim:
            print(f"Invalid dimension: expected {self.feature_dim}, got {len(features)}")
            return False
        
        # Check for NaN or Inf values
        for i, val in enumerate(features):
            if not isinstance(val, (int, float)):
                print(f"Feature {i} is not numeric: {val}")
                return False
            if np.isnan(val) or np.isinf(val):
                print(f"Feature {i} contains NaN or Inf")
                return False
        
        return True
    
    @staticmethod
    def create_dummy_normalizer() -> "FeatureNormalizer":
        """
        Create a dummy normalizer for cases when scaler is not available
        Useful for testing without training phase
        
        Returns:
            FeatureNormalizer instance with identity transformation
        """
        normalizer = FeatureNormalizer()
        # Create a mock scaler that does nothing
        class DummyScaler:
            def transform(self, X):
                return X
        
        normalizer.scaler = DummyScaler()
        return normalizer
