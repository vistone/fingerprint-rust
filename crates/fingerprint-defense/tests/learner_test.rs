#[cfg(test)]
mod tests {
    use fingerprint_defense::database::FingerprintDatabase;
    use fingerprint_defense::learner::{ObservationStats, SelfLearningAnalyzer};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_learner_creation() {
        let db = Arc::new(FingerprintDatabase::new());
        let learner = SelfLearningAnalyzer::new(db);

        assert_eq!(learner.learning_threshold, 10);
        assert_eq!(learner.min_stability_score, 0.8);
    }

    #[test]
    fn test_threshold_setting() {
        let db = Arc::new(FingerprintDatabase::new());
        let mut learner = SelfLearningAnalyzer::new(db);

        learner.set_threshold(20);
        assert_eq!(learner.learning_threshold, 20);
    }

    #[test]
    fn test_stability_score_setting() {
        let db = Arc::new(FingerprintDatabase::new());
        let mut learner = SelfLearningAnalyzer::new(db);

        learner.set_min_stability_score(0.9);
        assert_eq!(learner.min_stability_score, 0.9);
    }

    #[test]
    fn test_stability_window_setting() {
        let db = Arc::new(FingerprintDatabase::new());
        let mut learner = SelfLearningAnalyzer::new(db);

        let new_window = Duration::from_secs(12 * 60 * 60); // 12小时
        learner.set_stability_window(new_window);
        assert_eq!(learner.stability_window, new_window);
    }

    #[test]
    fn test_observation_stats() {
        let db = Arc::new(FingerprintDatabase::new());
        let learner = SelfLearningAnalyzer::new(db);

        let stats = learner.get_observation_stats();
        assert_eq!(stats.total_observations, 0);
        assert_eq!(stats.stable_candidates, 0);
        assert_eq!(stats.learning_threshold, 10);
        assert_eq!(stats.min_stability_score, 0.8);
    }

    #[test]
    fn test_timestamp_functions() {
        use fingerprint_defense::learner::{current_unix_timestamp, timestamp_duration};

        let start = current_unix_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10)); // 短暂等待
        let end = current_unix_timestamp();

        let duration = timestamp_duration(start, end);
        assert!(duration.as_secs() <= 1); // 应该小于1秒
    }

    #[test]
    fn test_observation_structure() {
        use fingerprint_defense::learner::UnknownFingerprintObservation;
        use serde_json::json;

        let observation = UnknownFingerprintObservation {
            fingerprint_id: "test_id".to_string(),
            fingerprint_type: "tls".to_string(),
            first_seen: 1000,
            last_seen: 2000,
            observation_count: 5,
            stability_score: 0.75,
            features: json!({"test": "data"}),
        };

        assert_eq!(observation.fingerprint_id, "test_id");
        assert_eq!(observation.fingerprint_type, "tls");
        assert_eq!(observation.observation_count, 5);
        assert_eq!(observation.stability_score, 0.75);
    }
}
