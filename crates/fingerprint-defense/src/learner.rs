//! fingerprint自learnmodule
//!
//! automatic from observe to traffic in learn and Updatefingerprintsignature. 

use crate::database::FingerprintDatabase;
use crate::passive::PassiveAnalysisResult;
use dashmap::DashMap;
use std::sync::Arc;

use fingerprint_core::fingerprint::Fingerprint;

/// 自learnanalysiser
pub struct SelfLearningAnalyzer {
 #[allow(dead_code)] // will来will for storelearn to fingerprint
 db: Arc<FingerprintDatabase>,
 /// not知fingerprintobservecounter (fp_id -> count)
 observations: DashMap<String, u64>,
 /// learning thresholdvalue (observemultiple少timebackturnenterdatabase)
 learning_threshold: u64,
}

impl SelfLearningAnalyzer {
 /// Create a newlearnanalysiser
 pub fn new(db: Arc<FingerprintDatabase>) -> Self {
 Self {
 db,
 observations: DashMap::new(),
 learning_threshold: 10,
 }
 }

 /// processanalysisresult并learn
 pub fn process_result(&self, result: &PassiveAnalysisResult) {
 // respectivelyprocesseachlayerlevelfingerprint
 if let Some(tls) = &result.tls {
 // TLS itemfrontdirectlyobserve ID (JA4)
 self.observe(tls.id(), "tls");
 }

 if let Some(http) = &result.http {
 if http.signature.is_none() {
 self.observe(http.id(), "http");
 }
 }

 if let Some(tcp) = &result.tcp {
 if tcp.signature.is_none() {
 self.observe(tcp.id(), "tcp");
 }
 }
 }

 /// observe to anfingerprint
 fn observe(&self, fp_id: String, fp_type: &str) {
 if fp_id == "unknown" || fp_id.is_empty() {
 return;
 }

 let key = format!("{}:{}", fp_type, fp_id);

 // protectionpoint：limitobservelistsize, preventinsidesaveburst (DoS protection)
 const MAX_OBSERVATIONS: usize = 10000;
 if self.observations.len() >= MAX_OBSERVATIONS && !self.observations.contains_key(&key) {
 // Ifreach to uplimit and  is new key, thenignore
 return;
 }

 let mut count = self.observations.entry(key.clone()).or_insert(0);
 *count += 1;

 if *count >= self.learning_threshold {
 // reach to thresholdvalue, can in database in establishinitialstepentry
 // TODO: Extracttrait并store as pendingcoreallowsignature
 println!("[Learner] Detected stable unknown fingerprint: {}", key);
 }
 }

 /// settingslearning thresholdvalue
 pub fn set_threshold(&mut self, threshold: u64) {
 self.learning_threshold = threshold;
 }
}
