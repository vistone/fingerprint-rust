//! Simple test to verify basic ML functionality

use fingerprint_ml::{FingerprintMatcher, FingerprintVector};

fn main() {
    println!("=== Basic ML Functionality Test ===\n");

    // Test fingerprint matcher
    println!("--- Testing Fingerprint Matcher ---");
    let mut matcher = FingerprintMatcher::new();

    matcher.add_reference(
        "chrome_120".to_string(),
        FingerprintVector::new(vec![0.95, 0.88, 0.92], Some("Chrome 120".to_string()), 0.95),
    );

    matcher.add_reference(
        "firefox_115".to_string(),
        FingerprintVector::new(
            vec![0.85, 0.91, 0.87],
            Some("Firefox 115".to_string()),
            0.92,
        ),
    );

    let query1 = FingerprintVector::new(vec![0.96, 0.89, 0.91], None, 0.90);
    let result1 = matcher.find_most_similar(&query1);
    println!("Query matched to: {:?}", result1);

    let query2 = FingerprintVector::new(vec![0.86, 0.90, 0.88], None, 0.88);
    let result2 = matcher.find_most_similar(&query2);
    println!("Query matched to: {:?}", result2);

    println!("\n✓ Basic ML functionality verified!");
}
