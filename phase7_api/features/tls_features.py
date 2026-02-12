"""
TLS Feature Extraction for Browser Fingerprinting
Extracts 12 TLS features from handshake data
"""

from typing import List, Dict, Any, Tuple
import struct


class TLSFeatureExtractor:
    """Extract TLS features from browser fingerprints"""
    
    # Common TLS versions
    TLS_VERSIONS = {
        0x0303: "TLS1.2",  # TLS 1.2
        0x0304: "TLS1.3",  # TLS 1.3
    }
    
    # Common cipher suites
    COMMON_CIPHERS = {
        0x1301: "TLS_AES_128_GCM_SHA256",
        0x1302: "TLS_AES_256_GCM_SHA384",
        0x1303: "TLS_CHACHA20_POLY1305_SHA256",
        0x002f: "SSL_RSA_WITH_AES_128_CBC_SHA",
        0x0035: "SSL_RSA_WITH_AES_256_CBC_SHA",
        0x003c: "SSL_RSA_WITH_AES_128_CBC_SHA256",
        0x003d: "SSL_RSA_WITH_AES_256_CBC_SHA256",
    }
    
    # Common elliptic curves
    COMMON_CURVES = {
        23: "secp256r1",
        24: "secp384r1",
        25: "secp521r1",
        29: "x25519",
    }
    
    def extract_tls_version(self, tls_data: bytes) -> float:
        """
        Extract TLS version from ClientHello
        Returns: Version number (e.g., 1.2 or 1.3)
        """
        if len(tls_data) < 4:
            return 0.0
        
        version = struct.unpack(">H", tls_data[1:3])[0]
        
        if version == 0x0303:
            return 1.2
        elif version == 0x0304:
            return 1.3
        else:
            return float(version) / 256.0
    
    def extract_cipher_suites(self, tls_data: bytes) -> Tuple[int, List[int]]:
        """
        Extract cipher suites from ClientHello
        Returns: (num_ciphers, first_cipher)
        """
        if len(tls_data) < 43:
            return (0, 0)
        
        # Skip to cipher suites length (2 bytes at offset 43)
        cipher_length = struct.unpack(">H", tls_data[43:45])[0]
        num_ciphers = cipher_length // 2
        
        # Get first cipher
        first_cipher = 0
        if len(tls_data) >= 47:
            first_cipher = struct.unpack(">H", tls_data[45:47])[0]
        
        return (num_ciphers, first_cipher)
    
    def extract_extensions(self, tls_data: bytes) -> Tuple[int, List[str]]:
        """
        Extract extension count and types from ClientHello
        Returns: (num_extensions, extension_list)
        """
        if len(tls_data) < 43:
            return (0, [])
        
        # Find extensions offset (after cipher suites)
        cipher_length = struct.unpack(">H", tls_data[43:45])[0]
        ext_offset = 45 + cipher_length + 1  # +1 for compression methods length
        
        if ext_offset >= len(tls_data):
            return (0, [])
        
        # Extension list length
        if ext_offset + 2 > len(tls_data):
            return (0, [])
        
        ext_list_length = struct.unpack(">H", tls_data[ext_offset:ext_offset+2])[0]
        extensions = []
        
        # Parse extensions
        pos = ext_offset + 2
        while pos + 4 <= ext_offset + 2 + ext_list_length and len(extensions) < 20:
            ext_type = struct.unpack(">H", tls_data[pos:pos+2])[0]
            ext_length = struct.unpack(">H", tls_data[pos+2:pos+4])[0]
            extensions.append(f"ext_{ext_type}")
            pos += 4 + ext_length
        
        return (len(extensions), extensions)
    
    def extract_curves(self, tls_data: bytes) -> Tuple[int, List[str]]:
        """
        Extract elliptic curves (supported_groups extension)
        Returns: (num_curves, curve_list)
        """
        curves = []
        
        # Simplified: look for supported_groups extension (type 10)
        try:
            if b"\x00\x0a" in tls_data:  # Extension type 10
                idx = tls_data.index(b"\x00\x0a")
                if idx + 6 < len(tls_data):
                    num_curves = struct.unpack(">H", tls_data[idx+6:idx+8])[0] // 2
                    for i in range(min(num_curves, 10)):
                        curve_id = struct.unpack(">H", tls_data[idx+8+i*2:idx+10+i*2])[0]
                        curve_name = self.COMMON_CURVES.get(curve_id, f"curve_{curve_id}")
                        curves.append(curve_name)
        except:
            pass
        
        return (len(curves), curves)
    
    def extract_signature_algorithms(self, tls_data: bytes) -> Tuple[int, List[str]]:
        """
        Extract signature algorithms from extension
        Returns: (num_algorithms, algorithm_list)
        """
        algorithms = []
        
        # Simplified: look for signature_algorithms extension (type 13)
        try:
            if b"\x00\x0d" in tls_data:  # Extension type 13
                idx = tls_data.index(b"\x00\x0d")
                if idx + 6 < len(tls_data):
                    num_algs = struct.unpack(">H", tls_data[idx+6:idx+8])[0] // 2
                    for i in range(min(num_algs, 10)):
                        alg_id = struct.unpack(">H", tls_data[idx+8+i*2:idx+10+i*2])[0]
                        algorithms.append(f"sig_{alg_id}")
        except:
            pass
        
        return (len(algorithms), algorithms)
    
    def extract(self, tls_data: bytes) -> Dict[str, Any]:
        """
        Extract all TLS features from ClientHello data
        
        Returns 12 features:
        1. tls_version
        2. cipher_suite_count
        3. first_cipher_suite
        4. extension_count
        5. curves_count
        6. signature_algs_count
        7-12. Placeholder numeric features
        """
        features = {}
        
        # Extract individual features
        features["tls_version"] = self.extract_tls_version(tls_data)
        cipher_count, first_cipher = self.extract_cipher_suites(tls_data)
        features["cipher_suite_count"] = cipher_count
        features["first_cipher_suite"] = first_cipher
        
        ext_count, _ = self.extract_extensions(tls_data)
        features["extension_count"] = ext_count
        
        curves_count, _ = self.extract_curves(tls_data)
        features["curves_count"] = curves_count
        
        sig_algs_count, _ = self.extract_signature_algorithms(tls_data)
        features["signature_algs_count"] = sig_algs_count
        
        # Placeholder numeric features
        features["feature_7"] = 0.0
        features["feature_8"] = 0.0
        features["feature_9"] = 0.0
        features["feature_10"] = 0.0
        features["feature_11"] = 0.0
        features["feature_12"] = 0.0
        
        return features
    
    def extract_feature_vector(self, tls_data: bytes) -> List[float]:
        """
        Extract TLS features as a vector for ML models
        Returns 12-element feature vector
        """
        features = self.extract(tls_data)
        
        vector = [
            features.get("tls_version", 0.0),
            features.get("cipher_suite_count", 0.0),
            features.get("first_cipher_suite", 0.0),
            features.get("extension_count", 0.0),
            features.get("curves_count", 0.0),
            features.get("signature_algs_count", 0.0),
            features.get("feature_7", 0.0),
            features.get("feature_8", 0.0),
            features.get("feature_9", 0.0),
            features.get("feature_10", 0.0),
            features.get("feature_11", 0.0),
            features.get("feature_12", 0.0),
        ]
        
        return vector
