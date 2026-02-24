//! # fingerprint-config
//!
//! Unified configuration management system for the fingerprint-rust project.
//! Provides centralized configuration handling with support for multiple sources
//! and runtime configuration updates.
//!
//! ## Features
//!
//! - ✅ **Multi-source Configuration**: File system, environment variables, remote sources
//! - ✅ **Hot Reload**: Runtime configuration updates without restart
//! - ✅ **Validation**: Type-safe configuration with validation rules
//! - ✅ **Hierarchical Structure**: Support for nested configuration sections
//! - ✅ **Caching**: Efficient configuration access with thread-safe caching
//!
//! ## Configuration Sources Priority
//!
//! 1. **Environment Variables** (highest priority)
//! 2. **Remote Configuration** (if enabled)
//! 3. **Local Configuration Files**
//! 4. **Built-in Defaults** (lowest priority)

use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::path::PathBuf;

/// Configuration management error types
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    #[error("Environment variable error: {0}")]
    EnvError(String),
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("Remote configuration error: {0}")]
    RemoteError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Main configuration manager
pub struct ConfigManager {
    /// Cached configuration values
    cache: DashMap<String, serde_json::Value>,
    
    /// Configuration sources
    sources: RwLock<Vec<ConfigSource>>,
    
    /// Validation rules
    validators: RwLock<HashMap<String, Box<dyn Validator>>>,
    
    /// Hot reload watchers
    watchers: RwLock<Vec<Box<dyn ConfigWatcher>>>,
}

/// Configuration source trait
pub trait ConfigSource: Send + Sync {
    fn name(&self) -> &str;
    fn load(&self) -> Result<serde_json::Value, ConfigError>;
    fn priority(&self) -> u32;
}

/// Configuration validator trait
pub trait Validator: Send + Sync {
    fn validate(&self, value: &serde_json::Value) -> Result<(), ConfigError>;
}

/// Configuration watcher trait for hot reload
pub trait ConfigWatcher: Send + Sync {
    fn watch(&self, callback: Box<dyn Fn() + Send + Sync>);
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            sources: RwLock::new(vec![]),
            validators: RwLock::new(HashMap::new()),
            watchers: RwLock::new(vec![]),
        }
    }

    /// Add a configuration source
    pub fn add_source(&self, source: Box<dyn ConfigSource>) {
        let mut sources = self.sources.write();
        sources.push(source);
        sources.sort_by_key(|s| std::u32::MAX - s.priority()); // Higher priority first
    }

    /// Add a validator for a configuration path
    pub fn add_validator(&self, path: String, validator: Box<dyn Validator>) {
        self.validators.write().insert(path, validator);
    }

    /// Load all configuration sources
    pub fn load(&self) -> Result<(), ConfigError> {
        let sources = self.sources.read();
        
        for source in sources.iter() {
            match source.load() {
                Ok(config) => {
                    self.merge_config(config)?;
                }
                Err(e) => {
                    log::warn!("Failed to load config source {}: {}", source.name(), e);
                }
            }
        }
        
        self.validate()?;
        Ok(())
    }

    /// Get a configuration value by path
    pub fn get<T>(&self, path: &str) -> Result<T, ConfigError>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(value) = self.cache.get(path) {
            serde_json::from_value(value.clone())
                .map_err(|e| ConfigError::ParseError(format!("Failed to deserialize {}: {}", path, e)))
        } else {
            Err(ConfigError::FileNotFound(path.to_string()))
        }
    }

    /// Set a configuration value
    pub fn set<T>(&self, path: &str, value: T) -> Result<(), ConfigError>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)
            .map_err(|e| ConfigError::ParseError(format!("Failed to serialize {}: {}", path, e)))?;
        
        self.cache.insert(path.to_string(), json_value);
        self.validate_path(path)?;
        Ok(())
    }

    /// Merge configuration from a JSON value
    fn merge_config(&self, config: serde_json::Value) -> Result<(), ConfigError> {
        self.flatten_and_cache("", &config);
        Ok(())
    }

    /// Flatten nested configuration and cache individual values
    fn flatten_and_cache(&self, prefix: &str, value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    self.flatten_and_cache(&new_prefix, val);
                }
            }
            _ => {
                self.cache.insert(prefix.to_string(), value.clone());
            }
        }
    }

    /// Validate all configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        let validators = self.validators.read();
        
        for (path, validator) in validators.iter() {
            if let Some(value) = self.cache.get(path) {
                validator.validate(&value)?;
            }
        }
        
        Ok(())
    }

    /// Validate a specific configuration path
    fn validate_path(&self, path: &str) -> Result<(), ConfigError> {
        let validators = self.validators.read();
        
        if let Some(validator) = validators.get(path) {
            if let Some(value) = self.cache.get(path) {
                validator.validate(&value)?;
            }
        }
        
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

// File system configuration source
#[cfg(feature = "file-system")]
pub mod file_source {
    use super::*;
    use std::fs;
    
    pub struct FileConfigSource {
        pub path: PathBuf,
        pub format: ConfigFormat,
        pub priority: u32,
    }
    
    #[derive(Debug, Clone)]
    pub enum ConfigFormat {
        Json,
        Toml,
        Yaml,
    }
    
    impl ConfigSource for FileConfigSource {
        fn name(&self) -> &str {
            "file"
        }
        
        fn load(&self) -> Result<serde_json::Value, ConfigError> {
            let content = fs::read_to_string(&self.path)?;
            
            let value = match self.format {
                ConfigFormat::Json => serde_json::from_str(&content)?,
                ConfigFormat::Toml => {
                    let toml_value: toml::Value = toml::from_str(&content)?;
                    serde_json::to_value(toml_value)?
                }
                ConfigFormat::Yaml => {
                    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
                    serde_json::to_value(yaml_value)?
                }
            };
            
            Ok(value)
        }
        
        fn priority(&self) -> u32 {
            self.priority
        }
    }
}

// Environment variable configuration source
#[cfg(feature = "environment")]
pub mod env_source {
    use super::*;
    
    pub struct EnvConfigSource {
        pub prefix: String,
        pub priority: u32,
    }
    
    impl ConfigSource for EnvConfigSource {
        fn name(&self) -> &str {
            "environment"
        }
        
        fn load(&self) -> Result<serde_json::Value, ConfigError> {
            let mut config_map = serde_json::Map::new();
            
            for (key, value) in std::env::vars() {
                if key.starts_with(&self.prefix) {
                    let config_key = key[self.prefix.len()..].to_lowercase();
                    config_map.insert(config_key, serde_json::Value::String(value));
                }
            }
            
            Ok(serde_json::Value::Object(config_map))
        }
        
        fn priority(&self) -> u32 {
            self.priority
        }
    }
}

// Built-in default configuration
pub mod defaults {
    use super::*;
    
    pub struct DefaultConfigSource {
        pub config: serde_json::Value,
        pub priority: u32,
    }
    
    impl ConfigSource for DefaultConfigSource {
        fn name(&self) -> &str {
            "defaults"
        }
        
        fn load(&self) -> Result<serde_json::Value, ConfigError> {
            Ok(self.config.clone())
        }
        
        fn priority(&self) -> u32 {
            self.priority
        }
    }
}

// Predefined validators
pub mod validators {
    use super::*;
    
    /// Range validator for numeric values
    pub struct RangeValidator {
        pub min: Option<f64>,
        pub max: Option<f64>,
    }
    
    impl Validator for RangeValidator {
        fn validate(&self, value: &serde_json::Value) -> Result<(), ConfigError> {
            if let Some(num) = value.as_f64() {
                if let Some(min) = self.min {
                    if num < min {
                        return Err(ConfigError::ValidationError(
                            format!("Value {} is below minimum {}", num, min)
                        ));
                    }
                }
                if let Some(max) = self.max {
                    if num > max {
                        return Err(ConfigError::ValidationError(
                            format!("Value {} is above maximum {}", num, max)
                        ));
                    }
                }
                Ok(())
            } else {
                Err(ConfigError::ValidationError("Expected numeric value".to_string()))
            }
        }
    }
    
    /// Enum validator for string values
    pub struct EnumValidator {
        pub allowed_values: Vec<String>,
    }
    
    impl Validator for EnumValidator {
        fn validate(&self, value: &serde_json::Value) -> Result<(), ConfigError> {
            if let Some(s) = value.as_str() {
                if self.allowed_values.contains(&s.to_string()) {
                    Ok(())
                } else {
                    Err(ConfigError::ValidationError(
                        format!("Value '{}' not in allowed values: {:?}", s, self.allowed_values)
                    ))
                }
            } else {
                Err(ConfigError::ValidationError("Expected string value".to_string()))
            }
        }
    }
    
    /// Required validator
    pub struct RequiredValidator;
    
    impl Validator for RequiredValidator {
        fn validate(&self, value: &serde_json::Value) -> Result<(), ConfigError> {
            if value.is_null() {
                Err(ConfigError::ValidationError("Value is required".to_string()))
            } else {
                Ok(())
            }
        }
    }
}

// Global configuration instance
static CONFIG_MANAGER: std::sync::OnceLock<Arc<ConfigManager>> = std::sync::OnceLock::new();

/// Get the global configuration manager instance
pub fn get_config_manager() -> Arc<ConfigManager> {
    CONFIG_MANAGER.get_or_init(|| {
        let manager = ConfigManager::new();
        
        // Add default configuration
        let default_config = serde_json::json!({
            "core": {
                "log_level": "info",
                "max_connections": 1000,
                "timeout_seconds": 30
            },
            "tls": {
                "min_version": "TLSv1_2",
                "cipher_suites": ["TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"],
                "enable_ja4_plus": true
            },
            "http": {
                "user_agent": "fingerprint-rust/2.1.0",
                "max_redirects": 5,
                "enable_quic": true
            },
            "defense": {
                "enable_learning": true,
                "anomaly_threshold": 0.8,
                "block_suspicious": false
            }
        });
        
        manager.add_source(Box::new(defaults::DefaultConfigSource {
            config: default_config,
            priority: 0,
        }));
        
        // Add validators
        manager.add_validator("core.max_connections".to_string(), 
                             Box::new(validators::RangeValidator { min: Some(1.0), max: Some(10000.0) }));
        manager.add_validator("core.log_level".to_string(),
                             Box::new(validators::EnumValidator { 
                                 allowed_values: vec!["trace".to_string(), "debug".to_string(), 
                                                    "info".to_string(), "warn".to_string(), "error".to_string()] 
                             }));
        manager.add_validator("defense.anomaly_threshold".to_string(),
                             Box::new(validators::RangeValidator { min: Some(0.0), max: Some(1.0) }));
        
        Arc::new(manager)
    }).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert_eq!(manager.cache.len(), 0);
    }
    
    #[tokio::test]
    async fn test_default_config_loading() {
        let manager = get_config_manager();
        manager.load().unwrap();
        
        let log_level: String = manager.get("core.log_level").unwrap();
        assert_eq!(log_level, "info");
        
        let max_conn: u32 = manager.get("core.max_connections").unwrap();
        assert_eq!(max_conn, 1000);
    }
    
    #[test]
    fn test_range_validator() {
        let validator = validators::RangeValidator { min: Some(1.0), max: Some(100.0) };
        
        // Valid values
        assert!(validator.validate(&serde_json::Value::Number(50.into())).is_ok());
        assert!(validator.validate(&serde_json::Value::Number(1.into())).is_ok());
        assert!(validator.validate(&serde_json::Value::Number(100.into())).is_ok());
        
        // Invalid values
        assert!(validator.validate(&serde_json::Value::Number(0.into())).is_err());
        assert!(validator.validate(&serde_json::Value::Number(101.into())).is_err());
    }
    
    #[test]
    fn test_enum_validator() {
        let validator = validators::EnumValidator {
            allowed_values: vec!["info".to_string(), "debug".to_string(), "error".to_string()],
        };
        
        // Valid values
        assert!(validator.validate(&serde_json::Value::String("info".to_string())).is_ok());
        assert!(validator.validate(&serde_json::Value::String("debug".to_string())).is_ok());
        
        // Invalid values
        assert!(validator.validate(&serde_json::Value::String("warning".to_string())).is_err());
        assert!(validator.validate(&serde_json::Value::Number(123.into())).is_err());
    }
}