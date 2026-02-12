"""
Inference Engine for 3-level Hierarchical Browser Fingerprint Classification
Implements Level 1 (Family), Level 2 (Version), and Level 3 (Variant) prediction
"""

from typing import Dict, Tuple, List, Optional, Any
from dataclasses import dataclass
import numpy as np

from .loader import ModelLoader


@dataclass
class PredictionResult:
    """Result of fingerprint identification"""
    family: str  # Browser family (e.g., Chrome, Firefox)
    family_confidence: float  # Confidence 0-1
    version: str  # Browser version (e.g., 120.0)
    version_confidence: float
    variant: str  # Variant (e.g., Standard, PSK, PQ)
    variant_confidence: float
    overall_confidence: float  # Average of all confidences
    raw_predictions: Dict[str, Any]  # Raw model outputs


class InferenceEngine:
    """
    Hierarchical inference engine for browser fingerprinting
    
    3-level classification:
    - Level 1: Predict browser family (11 classes)
    - Level 2: Predict version for the predicted family (variable classes)
    - Level 3: Predict variant for the predicted family (3 classes)
    """
    
    def __init__(self, models_dir: str = "./models"):
        """
        Initialize inference engine
        
        Args:
            models_dir: Path to directory containing trained models
        """
        self.model_loader = ModelLoader(models_dir)
        self.model_loader.load_all(lazy=False)
        
        self.inference_count = 0
        self.total_inference_time = 0.0  # In milliseconds
    
    def predict(self, features: List[float]) -> PredictionResult:
        """
        Perform complete 3-level hierarchical prediction
        
        Args:
            features: 53-dimensional normalized feature vector
            
        Returns:
            PredictionResult with family, version, and variant predictions
        """
        import time
        start_time = time.time()
        
        # Validate input
        if len(features) != 53:
            raise ValueError(f"Expected 53 features, got {len(features)}")
        
        # Convert to numpy array for sklearn
        features_array = np.array(features).reshape(1, -1)
        
        # Level 1: Predict family
        family_pred, family_confidence, family_label = self._predict_family(features_array)
        family_name = self.model_loader.get_family_name(family_pred)
        
        # Level 2: Predict version
        version_pred, version_confidence, version_label = self._predict_version(
            features_array, family_pred
        )
        
        # Level 3: Predict variant
        variant_pred, variant_confidence, variant_label = self._predict_variant(
            features_array, family_pred
        )
        
        # Calculate overall confidence
        overall_confidence = (family_confidence + version_confidence + variant_confidence) / 3.0
        
        # Record inference time
        elapsed = time.time() - start_time
        self.inference_count += 1
        self.total_inference_time += elapsed * 1000  # Convert to ms
        
        return PredictionResult(
            family=family_name,
            family_confidence=float(family_confidence),
            version=version_label,
            version_confidence=float(version_confidence),
            variant=variant_label,
            variant_confidence=float(variant_confidence),
            overall_confidence=float(overall_confidence),
            raw_predictions={
                "family_class": int(family_pred),
                "family_label": family_name,
                "family_score": float(family_confidence),
                "version_class": int(version_pred),
                "version_label": version_label,
                "version_score": float(version_confidence),
                "variant_class": int(variant_pred),
                "variant_label": variant_label,
                "variant_score": float(variant_confidence),
            }
        )
    
    def _predict_family(self, features: np.ndarray) -> Tuple[int, float, str]:
        """
        Level 1 prediction: Browser family
        
        Args:
            features: 1x53 feature array
            
        Returns:
            (predicted_class, confidence, label)
        """
        family_model = self.model_loader.get_model("family")
        if family_model is None:
            raise RuntimeError("Family model not loaded")
        
        # Predict
        prediction = family_model.predict(features)[0]
        
        # Get prediction probability
        if hasattr(family_model, "predict_proba"):
            proba = family_model.predict_proba(features)[0]
            confidence = float(np.max(proba))
        else:
            confidence = 1.0  # Default to full confidence if no proba available
        
        family_name = self.model_loader.get_family_name(int(prediction))
        
        return int(prediction), confidence, family_name
    
    def _predict_version(self, features: np.ndarray, family_index: int) -> Tuple[int, float, str]:
        """
        Level 2 prediction: Browser version for specific family
        
        Args:
            features: 1x53 feature array
            family_index: Family index from Level 1 prediction
            
        Returns:
            (predicted_class, confidence, label)
        """
        version_model = self.model_loader.get_model("version", family_index)
        if version_model is None:
            return 0, 0.5, "Unknown"
        
        # Predict
        prediction = version_model.predict(features)[0]
        
        # Get prediction probability
        if hasattr(version_model, "predict_proba"):
            proba = version_model.predict_proba(features)[0]
            confidence = float(np.max(proba))
        else:
            confidence = 1.0
        
        return int(prediction), confidence, f"v{prediction}"
    
    def _predict_variant(self, features: np.ndarray, family_index: int) -> Tuple[int, float, str]:
        """
        Level 3 prediction: Browser variant (Standard/PSK/PQ)
        
        Args:
            features: 1x53 feature array
            family_index: Family index from Level 1 prediction
            
        Returns:
            (predicted_class, confidence, label)
        """
        variant_model = self.model_loader.get_model("variant", family_index)
        if variant_model is None:
            return 0, 0.5, "Standard"
        
        # Variant mapping
        variant_names = {
            0: "Standard",
            1: "PSK",
            2: "PQ",
        }
        
        # Predict
        prediction = variant_model.predict(features)[0]
        
        # Get prediction probability
        if hasattr(variant_model, "predict_proba"):
            proba = variant_model.predict_proba(features)[0]
            confidence = float(np.max(proba))
        else:
            confidence = 1.0
        
        variant_label = variant_names.get(int(prediction), "Unknown")
        
        return int(prediction), confidence, variant_label
    
    def batch_predict(self, features_list: List[List[float]]) -> List[PredictionResult]:
        """
        Perform batch predictions
        
        Args:
            features_list: List of 53-dimensional feature vectors
            
        Returns:
            List of PredictionResult objects
        """
        results = []
        for features in features_list:
            try:
                result = self.predict(features)
                results.append(result)
            except Exception as e:
                print(f"Error in batch prediction: {e}")
                results.append(None)
        
        return results
    
    def get_statistics(self) -> Dict[str, Any]:
        """
        Get inference engine statistics
        
        Returns:
            Dictionary with inference statistics
        """
        avg_latency = (
            self.total_inference_time / self.inference_count
            if self.inference_count > 0
            else 0.0
        )
        
        return {
            "total_inferences": self.inference_count,
            "total_inference_time_ms": self.total_inference_time,
            "average_latency_ms": avg_latency,
            "throughput_samples_per_sec": 1000.0 / avg_latency if avg_latency > 0 else 0,
            "model_status": self.model_loader.get_status(),
        }
