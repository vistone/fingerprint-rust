# fingerprint-defense

Browser fingerprint defense system with passive detection and active protection capabilities.

## Features

### 🛡️ Passive Detection
- Network-level fingerprint recognition
- JA4+ signature analysis
- Behavioral anomaly detection
- Storage access monitoring

### 🎯 Active Protection
- Canvas fingerprint obfuscation
- Audio context noise injection
- HTTP header randomization
- Timing attack mitigation

### 📊 Machine Learning
- Self-learning fingerprint database
- Adaptive threat scoring
- Automated signature generation

## Usage

```rust
use fingerprint_defense::{
    AnomalyDetector,
    CanvasNoiseGenerator,
    StorageAnalyzer,
    SelfLearningAnalyzer,
    FingerprintDatabase
};
use std::sync::Arc;

// Initialize components
let db = FingerprintDatabase::new_in_memory();
let learner = SelfLearningAnalyzer::new(Arc::new(db));
let noise_gen = CanvasNoiseGenerator::new(0.3);
let storage_analyzer = StorageAnalyzer::new();

// Monitor storage access
storage_analyzer.record_local_storage_access("tracking_key");
if storage_analyzer.detect_local_storage_fingerprinting() {
    println!("Potential fingerprinting detected!");
}

// Generate anti-fingerprinting noise
let canvas_noise = noise_gen.generate_canvas_noise();
let webgl_params = noise_gen.generate_webgl_noise();

// Process network traffic
let detector = AnomalyDetector::new();
// ... process traffic data

// Self-learning capabilities
let stats = learner.get_observation_stats();
println!("Learning progress: {} stable candidates", stats.stable_candidates);
```

## Modules

- **`anomaly`** - Behavioral anomaly detection
- **`api_noise`** - Canvas and audio noise generation  
- **`database`** - Fingerprint storage and management
- **`hunting`** - Threat hunting and honeypot deployment
- **`learner`** - Self-learning fingerprint analysis
- **`passive`** - Passive fingerprint recognition
- **`storage`** - Storage-based fingerprinting detection
- **`timing`** - Timing attack protection

## Testing

```bash
cargo test -p fingerprint-defense
```

## License

BSD-3-Clause