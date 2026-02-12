"""
Model Loader for Browser Fingerprint Classification Models
Loads 18 trained models from Phase 7.3
"""

from typing import Dict, Optional, Any
import pickle
from pathlib import Path
import numpy as np


class ModelLoader:
    """Load and cache trained classification models"""
    
    # Model file names
    FAMILY_MODEL = "family_model.pkl"
    VERSION_MODELS = "version_models.pkl"
    VARIANT_MODELS = "variant_models.pkl"
    SCALER = "scaler.pkl"
    VERSION_ENCODERS = "version_encoders.pkl"
    FEATURE_INFO = "feature_info.json"
    
    def __init__(self, models_dir: str = "./models"):
        """
        Initialize model loader with models directory
        
        Args:
            models_dir: Path to directory containing trained models
        """
        self.models_dir = Path(models_dir)
        self.models_cache: Dict[str, Any] = {}
        self.loaded_models: Dict[str, bool] = {
            "family_classifier": False,
            "version_classifiers": False,
            "variant_classifiers": False,
            "scaler": False,
            "encoders": False,
        }
        
        # Browser family mapping (indices 0-10)
        self.family_names = [
            "Chrome",          # 0
            "Firefox",         # 1
            "Safari",          # 2
            "OkHttp",          # 3
            "Opera",           # 4
            "Cloudflare",      # 5
            "Confirmed",       # 6
            "Mesh",            # 7
            "MMS",             # 8
            "Nike",            # 9
            "Zalando",         # 10
        ]
    
    def load_all(self, lazy: bool = False) -> bool:
        """
        Load all trained models
        
        Args:
            lazy: If True, only load when needed
            
        Returns:
            True if all models loaded successfully, False otherwise
        """
        if lazy:
            return True
        
        success = True
        success &= self.load_family_classifier()
        success &= self.load_version_classifiers()
        success &= self.load_variant_classifiers()
        success &= self.load_scaler()
        success &= self.load_encoders()
        
        return success
    
    def load_family_classifier(self) -> bool:
        """
        Load Level 1 family classifier (single model)
        
        Returns:
            True if loaded successfully
        """
        try:
            model_path = self.models_dir / self.FAMILY_MODEL
            if not model_path.exists():
                print(f"Warning: {self.FAMILY_MODEL} not found at {model_path}")
                return False
            
            with open(model_path, "rb") as f:
                self.models_cache["family_classifier"] = pickle.load(f)
            
            self.loaded_models["family_classifier"] = True
            print(f"✓ Family classifier loaded ({model_path.stat().st_size / 1024:.1f} KB)")
            return True
        except Exception as e:
            print(f"Error loading family classifier: {e}")
            return False
    
    def load_version_classifiers(self) -> bool:
        """
        Load Level 2 version classifiers (11 models, one per family)
        
        Returns:
            True if loaded successfully
        """
        try:
            model_path = self.models_dir / self.VERSION_MODELS
            if not model_path.exists():
                print(f"Warning: {self.VERSION_MODELS} not found at {model_path}")
                return False
            
            with open(model_path, "rb") as f:
                version_models = pickle.load(f)
            
            self.models_cache["version_classifiers"] = version_models
            self.loaded_models["version_classifiers"] = True
            
            num_families = len(version_models) if isinstance(version_models, dict) else version_models.__len__()
            print(f"✓ Version classifiers loaded ({num_families} families, {model_path.stat().st_size / 1024 / 1024:.1f} MB)")
            return True
        except Exception as e:
            print(f"Error loading version classifiers: {e}")
            return False
    
    def load_variant_classifiers(self) -> bool:
        """
        Load Level 3 variant classifiers (6 models)
        
        Returns:
            True if loaded successfully
        """
        try:
            model_path = self.models_dir / self.VARIANT_MODELS
            if not model_path.exists():
                print(f"Warning: {self.VARIANT_MODELS} not found at {model_path}")
                return False
            
            with open(model_path, "rb") as f:
                self.models_cache["variant_classifiers"] = pickle.load(f)
            
            self.loaded_models["variant_classifiers"] = True
            print(f"✓ Variant classifiers loaded ({model_path.stat().st_size / 1024:.1f} KB)")
            return True
        except Exception as e:
            print(f"Error loading variant classifiers: {e}")
            return False
    
    def load_scaler(self) -> bool:
        """
        Load feature scaler for normalization
        
        Returns:
            True if loaded successfully
        """
        try:
            model_path = self.models_dir / self.SCALER
            if not model_path.exists():
                print(f"Warning: {self.SCALER} not found at {model_path}")
                return False
            
            with open(model_path, "rb") as f:
                self.models_cache["scaler"] = pickle.load(f)
            
            self.loaded_models["scaler"] = True
            print(f"✓ Feature scaler loaded ({model_path.stat().st_size / 1024:.2f} KB)")
            return True
        except Exception as e:
            print(f"Error loading scaler: {e}")
            return False
    
    def load_encoders(self) -> bool:
        """
        Load label encoders for version decoding
        
        Returns:
            True if loaded successfully
        """
        try:
            model_path = self.models_dir / self.VERSION_ENCODERS
            if not model_path.exists():
                print(f"Warning: {self.VERSION_ENCODERS} not found at {model_path}")
                return False
            
            with open(model_path, "rb") as f:
                self.models_cache["encoders"] = pickle.load(f)
            
            self.loaded_models["encoders"] = True
            print(f"✓ Label encoders loaded ({model_path.stat().st_size / 1024:.2f} KB)")
            return True
        except Exception as e:
            print(f"Error loading encoders: {e}")
            return False
    
    def get_model(self, model_type: str, family_index: Optional[int] = None) -> Optional[Any]:
        """
        Get a specific model from cache
        
        Args:
            model_type: "family", "version", or "variant"
            family_index: Family index for version/variant models
            
        Returns:
            Model object or None if not found
        """
        if model_type == "family":
            return self.models_cache.get("family_classifier")
        elif model_type == "version":
            version_models = self.models_cache.get("version_classifiers")
            if version_models and family_index is not None:
                if isinstance(version_models, dict):
                    return version_models.get(family_index)
                else:
                    # Assume it's a list-like object
                    try:
                        return version_models[family_index]
                    except (IndexError, TypeError):
                        return None
            return None
        elif model_type == "variant":
            variant_models = self.models_cache.get("variant_classifiers")
            if variant_models and family_index is not None:
                if isinstance(variant_models, dict):
                    return variant_models.get(family_index)
                else:
                    try:
                        return variant_models[family_index]
                    except (IndexError, TypeError):
                        return None
            return None
        
        return None
    
    def get_scaler(self) -> Optional[Any]:
        """Get the feature scaler"""
        return self.models_cache.get("scaler")
    
    def get_encoders(self) -> Optional[Any]:
        """Get the label encoders"""
        return self.models_cache.get("encoders")
    
    def get_status(self) -> Dict[str, bool]:
        """Get status of loaded models"""
        return self.loaded_models.copy()
    
    def get_family_name(self, family_index: int) -> str:
        """Get family name from index"""
        if 0 <= family_index < len(self.family_names):
            return self.family_names[family_index]
        return f"Unknown_{family_index}"
