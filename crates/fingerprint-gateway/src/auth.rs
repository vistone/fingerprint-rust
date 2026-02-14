//! Authentication and authorization
//!
//! Provides API key validation and tier determination

use crate::{error::GatewayError, models::QuotaTier};
use std::collections::HashMap;
use tracing::{debug, warn};

/// API key validator
pub struct ApiKeyValidator {
    keys: HashMap<String, ApiKeyInfo>,
}

/// API key information
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// API key
    pub key: String,

    /// Quota tier
    pub tier: QuotaTier,

    /// Owner/organization
    pub owner: String,

    /// Whether the key is active
    pub active: bool,
}

impl ApiKeyValidator {
    /// Create a new validator with default keys (for testing)
    pub fn new() -> Self {
        let mut keys = HashMap::new();

        // Add some default test keys
        keys.insert(
            "sk_test_demo123".to_string(),
            ApiKeyInfo {
                key: "sk_test_demo123".to_string(),
                tier: QuotaTier::Free,
                owner: "demo_user".to_string(),
                active: true,
            },
        );

        keys.insert(
            "sk_live_prod456".to_string(),
            ApiKeyInfo {
                key: "sk_live_prod456".to_string(),
                tier: QuotaTier::Pro,
                owner: "production_user".to_string(),
                active: true,
            },
        );

        keys.insert(
            "sk_enterprise_corp789".to_string(),
            ApiKeyInfo {
                key: "sk_enterprise_corp789".to_string(),
                tier: QuotaTier::Enterprise,
                owner: "enterprise_corp".to_string(),
                active: true,
            },
        );

        Self { keys }
    }

    /// Validate an API key and return its info
    pub fn validate(&self, api_key: &str) -> Result<ApiKeyInfo, GatewayError> {
        // Check if key exists
        if let Some(info) = self.keys.get(api_key) {
            if !info.active {
                warn!("Inactive API key used: {}", api_key);
                return Err(GatewayError::InvalidApiKey(
                    "API key is inactive".to_string(),
                ));
            }

            debug!("API key validated: {} (tier: {:?})", api_key, info.tier);
            Ok(info.clone())
        } else {
            // Fallback to prefix-based detection
            let tier = determine_tier_from_prefix(api_key);

            debug!(
                "API key not in database, using prefix detection: {} (tier: {:?})",
                api_key, tier
            );

            Ok(ApiKeyInfo {
                key: api_key.to_string(),
                tier,
                owner: "unknown".to_string(),
                active: true,
            })
        }
    }

    /// Add a new API key
    pub fn add_key(&mut self, info: ApiKeyInfo) {
        self.keys.insert(info.key.clone(), info);
    }

    /// Revoke an API key
    pub fn revoke_key(&mut self, api_key: &str) -> Result<(), GatewayError> {
        if let Some(info) = self.keys.get_mut(api_key) {
            info.active = false;
            debug!("API key revoked: {}", api_key);
            Ok(())
        } else {
            Err(GatewayError::InvalidApiKey(format!(
                "API key not found: {}",
                api_key
            )))
        }
    }
}

impl Default for ApiKeyValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Determine quota tier from API key prefix
fn determine_tier_from_prefix(api_key: &str) -> QuotaTier {
    if api_key.starts_with("sk_test_") {
        QuotaTier::Free
    } else if api_key.starts_with("sk_live_") {
        QuotaTier::Pro
    } else if api_key.starts_with("sk_enterprise_") {
        QuotaTier::Enterprise
    } else if api_key.starts_with("sk_partner_") {
        QuotaTier::Partner
    } else {
        QuotaTier::Free
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = ApiKeyValidator::new();
        assert!(!validator.keys.is_empty());
    }

    #[test]
    fn test_validate_existing_key() {
        let validator = ApiKeyValidator::new();
        let result = validator.validate("sk_test_demo123");
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.tier, QuotaTier::Free);
        assert!(info.active);
    }

    #[test]
    fn test_validate_unknown_key_with_prefix() {
        let validator = ApiKeyValidator::new();
        let result = validator.validate("sk_live_unknown");
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.tier, QuotaTier::Pro);
    }

    #[test]
    fn test_revoke_key() {
        let mut validator = ApiKeyValidator::new();
        let result = validator.revoke_key("sk_test_demo123");
        assert!(result.is_ok());

        let validate_result = validator.validate("sk_test_demo123");
        assert!(validate_result.is_err());
    }

    #[test]
    fn test_prefix_detection() {
        assert_eq!(determine_tier_from_prefix("sk_test_123"), QuotaTier::Free);
        assert_eq!(determine_tier_from_prefix("sk_live_456"), QuotaTier::Pro);
        assert_eq!(
            determine_tier_from_prefix("sk_enterprise_789"),
            QuotaTier::Enterprise
        );
        assert_eq!(
            determine_tier_from_prefix("sk_partner_abc"),
            QuotaTier::Partner
        );
        assert_eq!(determine_tier_from_prefix("invalid_key"), QuotaTier::Free);
    }

    #[test]
    fn test_add_key() {
        let mut validator = ApiKeyValidator::new();

        let new_key = ApiKeyInfo {
            key: "sk_test_new123".to_string(),
            tier: QuotaTier::Pro,
            owner: "new_user".to_string(),
            active: true,
        };

        validator.add_key(new_key);

        let result = validator.validate("sk_test_new123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tier, QuotaTier::Pro);
    }
}
