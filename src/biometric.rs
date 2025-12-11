//! Biometric validation and emotional state monitoring

use crate::crypto::KeyPair;
use crate::error::{ConsensusError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

/// Type of biometric reading
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BiometricType {
    /// Heart rate in BPM
    HeartRate,
    /// Stress level (0-100)
    StressLevel,
    /// Focus level (0-100)
    FocusLevel,
    /// Skin conductance
    SkinConductance,
    /// Skin temperature
    SkinTemperature,
}

/// Biometric reading from a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricReading {
    /// Device ID that generated this reading
    pub device_id: String,
    /// Type of biometric data
    pub biometric_type: BiometricType,
    /// Reading value
    pub value: f64,
    /// Quality score (0.0 - 1.0)
    pub quality: f64,
    /// Timestamp (Unix milliseconds)
    pub timestamp: u64,
    /// Optional metadata
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Emotional profile of a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalProfile {
    /// Current emotional score (0-100)
    pub emotional_score: u8,
    /// Trend direction
    pub trend: EmotionalTrend,
    /// Confidence in the score (0-100)
    pub confidence: u8,
    /// Last update timestamp
    pub last_updated: u64,
    /// Recent biometric readings
    pub recent_readings: Vec<BiometricReading>,
}

/// Trend in emotional score
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EmotionalTrend {
    /// Score is improving
    Improving,
    /// Score is stable
    Stable,
    /// Score is declining
    Declining,
}

/// Mock biometric device for testing
pub trait BiometricDevice: Send + Sync {
    /// Collect biometric readings
    fn collect_readings(&self) -> Result<Vec<BiometricReading>>;

    /// Get device ID
    fn device_id(&self) -> &str;

    /// Check if device is functioning
    fn is_healthy(&self) -> bool;
}

/// Validator with emotional monitoring
pub struct EmotionalValidator {
    /// Validator ID
    pub id: String,
    /// Cryptographic key pair
    pub key_pair: KeyPair,
    /// Current stake in POE tokens
    pub stake: Arc<RwLock<u64>>,
    /// Current balance in POE tokens
    pub balance: Arc<RwLock<u64>>,
    /// Is validator active
    pub is_active: Arc<RwLock<bool>>,
    /// Current emotional profile
    pub emotional_profile: Arc<RwLock<Option<EmotionalProfile>>>,
    /// Historical emotional scores
    score_history: Arc<RwLock<VecDeque<(u8, u64)>>>,
    /// Reputation score (0-100)
    pub reputation: Arc<RwLock<u8>>,
}

impl EmotionalValidator {
    /// Create a new emotional validator
    pub fn new(id: impl Into<String>, stake: u64) -> Result<Self> {
        let key_pair = KeyPair::generate()?;

        Ok(Self {
            id: id.into(),
            key_pair,
            stake: Arc::new(RwLock::new(stake)),
            balance: Arc::new(RwLock::new(0)),
            is_active: Arc::new(RwLock::new(true)),
            emotional_profile: Arc::new(RwLock::new(None)),
            score_history: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            reputation: Arc::new(RwLock::new(100)),
        })
    }

    /// Create validator from existing key pair
    pub fn from_keypair(id: impl Into<String>, stake: u64, key_pair: KeyPair) -> Self {
        Self {
            id: id.into(),
            key_pair,
            stake: Arc::new(RwLock::new(stake)),
            balance: Arc::new(RwLock::new(0)),
            is_active: Arc::new(RwLock::new(true)),
            emotional_profile: Arc::new(RwLock::new(None)),
            score_history: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            reputation: Arc::new(RwLock::new(100)),
        }
    }

    /// Update emotional state from biometric readings
    pub async fn update_emotional_state(&self, readings: Vec<BiometricReading>) -> Result<()> {
        if readings.is_empty() {
            return Err(ConsensusError::biometric_validation_failed(
                "No biometric readings provided",
            ));
        }

        let emotional_score = self.calculate_emotional_score(&readings)?;
        let trend = self.analyze_trend(emotional_score);
        let confidence = self.calculate_confidence(&readings);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let profile = EmotionalProfile {
            emotional_score,
            trend,
            confidence,
            last_updated: timestamp,
            recent_readings: readings,
        };

        *self.emotional_profile.write() = Some(profile);

        let mut history = self.score_history.write();
        history.push_back((emotional_score, timestamp));
        if history.len() > 100 {
            history.pop_front();
        }

        Ok(())
    }

    /// Calculate emotional score from biometric readings
    fn calculate_emotional_score(&self, readings: &[BiometricReading]) -> Result<u8> {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for reading in readings {
            let (score, weight) = match reading.biometric_type {
                BiometricType::HeartRate => {
                    let hr = reading.value;
                    let score = if (60.0..=80.0).contains(&hr) {
                        100.0
                    } else if (50.0..=100.0).contains(&hr) {
                        80.0
                    } else {
                        50.0
                    };
                    (score, reading.quality)
                }
                BiometricType::StressLevel => {
                    let stress = reading.value.clamp(0.0, 100.0);
                    let score = 100.0 - stress;
                    (score, reading.quality)
                }
                BiometricType::FocusLevel => {
                    let focus = reading.value.clamp(0.0, 100.0);
                    (focus, reading.quality)
                }
                _ => (75.0, reading.quality),
            };

            total_score += score * weight;
            total_weight += weight;
        }

        if total_weight == 0.0 {
            return Err(ConsensusError::biometric_validation_failed(
                "No valid readings with quality > 0",
            ));
        }

        let final_score = (total_score / total_weight).clamp(0.0, 100.0) as u8;
        Ok(final_score)
    }

    /// Analyze trend in emotional scores
    fn analyze_trend(&self, _current_score: u8) -> EmotionalTrend {
        let history = self.score_history.read();

        if history.len() < 3 {
            return EmotionalTrend::Stable;
        }

        let recent: Vec<_> = history.iter().rev().take(5).map(|(s, _)| *s).collect();

        let n = recent.len() as f64;
        let sum_x: f64 = (0..recent.len()).map(|i| i as f64).sum();
        let sum_y: f64 = recent.iter().map(|&s| s as f64).sum();
        let sum_xy: f64 = recent
            .iter()
            .enumerate()
            .map(|(i, &s)| i as f64 * s as f64)
            .sum();
        let sum_xx: f64 = (0..recent.len()).map(|i| (i * i) as f64).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);

        if slope > 2.0 {
            EmotionalTrend::Improving
        } else if slope < -2.0 {
            EmotionalTrend::Declining
        } else {
            EmotionalTrend::Stable
        }
    }

    /// Calculate confidence in the emotional score
    fn calculate_confidence(&self, readings: &[BiometricReading]) -> u8 {
        if readings.is_empty() {
            return 0;
        }

        let avg_quality = readings.iter().map(|r| r.quality).sum::<f64>() / readings.len() as f64;
        let quality_score = (avg_quality * 100.0) as u8;

        let unique_types: std::collections::HashSet<_> =
            readings.iter().map(|r| &r.biometric_type).collect();
        let multimodal_bonus = (unique_types.len() * 5).min(20) as u8;

        let timestamps: Vec<_> = readings.iter().map(|r| r.timestamp).collect();
        let time_span = timestamps.iter().max().unwrap() - timestamps.iter().min().unwrap();
        let temporal_bonus = if time_span < 5000 {
            10
        } else if time_span < 60000 {
            5
        } else {
            0
        };

        (quality_score + multimodal_bonus + temporal_bonus).min(100)
    }

    /// Get current emotional score
    pub fn get_emotional_score(&self) -> u8 {
        self.emotional_profile
            .read()
            .as_ref()
            .map(|p| p.emotional_score)
            .unwrap_or(0)
    }

    /// Get current emotional profile
    pub fn get_emotional_profile(&self) -> Option<EmotionalProfile> {
        self.emotional_profile.read().clone()
    }

    /// Check if validator is eligible for consensus
    pub fn is_eligible(&self, emotional_threshold: u8, minimum_stake: u64) -> bool {
        *self.is_active.read()
            && *self.stake.read() >= minimum_stake
            && self.get_emotional_score() >= emotional_threshold
    }

    /// Add reward
    pub fn add_reward(&self, amount: u64) {
        let mut balance = self.balance.write();
        *balance = balance.saturating_add(amount);
    }

    /// Apply slashing penalty
    pub fn apply_slashing(&self, amount: u64) {
        let mut stake = self.stake.write();
        *stake = stake.saturating_sub(amount);

        let mut reputation = self.reputation.write();
        let penalty = ((amount as f64 / *stake as f64) * 10.0).min(20.0) as u8;
        *reputation = reputation.saturating_sub(penalty);
    }

    /// Get validator ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get public key
    pub fn public_key_hex(&self) -> String {
        self.key_pair.public_key_hex()
    }

    /// Get current stake
    pub fn get_stake(&self) -> u64 {
        *self.stake.read()
    }

    /// Get current balance
    pub fn get_balance(&self) -> u64 {
        *self.balance.read()
    }

    /// Get reputation
    pub fn get_reputation(&self) -> u8 {
        *self.reputation.read()
    }

    /// Validate a block proposal
    ///
    /// Performs comprehensive validation including:
    /// - Block hash verification
    /// - Previous hash validation
    /// - Block height sequence check
    /// - Transaction hash verification
    /// - Merkle root validation
    /// - Timestamp reasonableness check
    pub fn validate_block(
        &self,
        block: &crate::types::Block,
        expected_previous_hash: &str,
        expected_height: u64,
    ) -> std::result::Result<(), String> {
        // 1. Verify block hash matches content
        if !block.verify_hash() {
            return Err("Block hash does not match content".to_string());
        }

        // 2. Verify previous hash
        if block.header.previous_hash != expected_previous_hash {
            return Err(format!(
                "Previous hash mismatch: expected {}, got {}",
                expected_previous_hash, block.header.previous_hash
            ));
        }

        // 3. Verify block height is sequential
        if block.header.height != expected_height {
            return Err(format!(
                "Block height mismatch: expected {}, got {}",
                expected_height, block.header.height
            ));
        }

        // 4. Verify all transaction hashes
        for (i, tx) in block.transactions.iter().enumerate() {
            if !tx.verify_hash() {
                return Err(format!("Transaction {} has invalid hash", i));
            }
        }

        // 5. Verify merkle root
        let calculated_merkle = crate::types::Block::calculate_merkle_root(&block.transactions);
        if calculated_merkle != block.header.merkle_root {
            return Err(format!(
                "Merkle root mismatch: expected {}, got {}",
                calculated_merkle, block.header.merkle_root
            ));
        }

        // 6. Verify timestamp is reasonable (not in future, not too old)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Block timestamp should not be more than 5 seconds in the future
        if block.header.timestamp > now + 5000 {
            return Err("Block timestamp is too far in the future".to_string());
        }

        // Block timestamp should not be more than 1 hour old
        if block.header.timestamp < now.saturating_sub(3600000) {
            return Err("Block timestamp is too old (>1 hour)".to_string());
        }

        // 7. Verify proposer is in the validator ID field
        if block.header.validator_id.is_empty() {
            return Err("Block has no validator ID".to_string());
        }

        // 8. Verify block signature
        match block.verify_signature() {
            Ok(true) => {}
            Ok(false) => return Err("Block signature verification failed".to_string()),
            Err(e) => return Err(format!("Block signature error: {}", e)),
        }

        // 9. Verify all transaction signatures
        for (i, tx) in block.transactions.iter().enumerate() {
            match tx.verify_signature() {
                Ok(true) => {}
                Ok(false) => {
                    return Err(format!("Transaction {} signature verification failed", i))
                }
                Err(e) => return Err(format!("Transaction {} signature error: {}", i, e)),
            }
        }

        Ok(())
    }
}

/// Production-quality biometric simulator for testing
pub struct BiometricSimulator {
    device_id: String,
    validator_seed: u64,
}

impl BiometricSimulator {
    /// Create a new biometric simulator
    pub fn new(device_id: String, validator_id: &str) -> Self {
        let validator_seed = validator_id
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

        Self {
            device_id,
            validator_seed,
        }
    }

    /// Generate realistic heart rate
    fn generate_heart_rate(&self, timestamp: u64) -> f64 {
        let baseline = 60.0 + (self.validator_seed % 25) as f64;
        let time_of_day =
            (timestamp % (24 * 60 * 60 * 1000)) as f64 / (24.0 * 60.0 * 60.0 * 1000.0);

        let circadian_factor =
            1.0 + 0.15 * (2.0 * std::f64::consts::PI * (time_of_day - 0.25)).sin();
        let stress_variation =
            0.9 + 0.2 * ((self.validator_seed as f64 + timestamp as f64 / 300000.0).sin());

        baseline * circadian_factor * stress_variation
    }

    /// Generate realistic stress level
    fn generate_stress_level(&self, timestamp: u64) -> f64 {
        let base_stress = (self.validator_seed % 40) as f64;
        let time_of_day =
            (timestamp % (24 * 60 * 60 * 1000)) as f64 / (24.0 * 60.0 * 60.0 * 1000.0);

        let work_factor = if (0.375..=0.75).contains(&time_of_day) {
            1.3
        } else {
            0.8
        };

        (base_stress * work_factor).min(100.0)
    }

    /// Generate realistic focus level
    fn generate_focus_level(&self, timestamp: u64) -> f64 {
        let base_focus = 60.0 + ((self.validator_seed % 30) as f64);
        let time_of_day =
            (timestamp % (24 * 60 * 60 * 1000)) as f64 / (24.0 * 60.0 * 60.0 * 1000.0);

        let circadian_focus = 0.7
            + 0.3
                * f64::max(
                    (2.0 * std::f64::consts::PI * (time_of_day - 0.25)).sin(),
                    (2.0 * std::f64::consts::PI * (time_of_day - 0.7)).sin(),
                );

        (base_focus * circadian_focus).min(100.0)
    }
}

impl BiometricDevice for BiometricSimulator {
    fn collect_readings(&self) -> Result<Vec<BiometricReading>> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Ok(vec![
            BiometricReading {
                device_id: format!("{}_heart", self.device_id),
                biometric_type: BiometricType::HeartRate,
                value: self.generate_heart_rate(timestamp),
                quality: 0.85 + ((self.validator_seed % 15) as f64 / 100.0),
                timestamp,
                metadata: None,
            },
            BiometricReading {
                device_id: format!("{}_stress", self.device_id),
                biometric_type: BiometricType::StressLevel,
                value: self.generate_stress_level(timestamp),
                quality: 0.85 + ((self.validator_seed % 15) as f64 / 100.0),
                timestamp: timestamp + 100,
                metadata: None,
            },
            BiometricReading {
                device_id: format!("{}_focus", self.device_id),
                biometric_type: BiometricType::FocusLevel,
                value: self.generate_focus_level(timestamp),
                quality: 0.85 + ((self.validator_seed % 15) as f64 / 100.0),
                timestamp: timestamp + 200,
                metadata: None,
            },
        ])
    }

    fn device_id(&self) -> &str {
        &self.device_id
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validator_creation() {
        let validator = EmotionalValidator::new("test-validator", 10000).unwrap();
        assert_eq!(validator.get_stake(), 10000);
        assert_eq!(validator.get_emotional_score(), 0);
    }

    #[tokio::test]
    async fn test_emotional_state_update() {
        let validator = EmotionalValidator::new("test-validator", 10000).unwrap();
        let simulator = BiometricSimulator::new("device1".to_string(), "test-validator");

        let readings = simulator.collect_readings().unwrap();
        validator.update_emotional_state(readings).await.unwrap();

        let score = validator.get_emotional_score();
        assert!(score > 0);
        assert!(score <= 100);
    }

    #[tokio::test]
    async fn test_eligibility_check() {
        let validator = EmotionalValidator::new("test-validator", 10000).unwrap();
        let simulator = BiometricSimulator::new("device1".to_string(), "test-validator");

        let readings = simulator.collect_readings().unwrap();
        validator.update_emotional_state(readings).await.unwrap();

        assert!(validator.is_eligible(50, 10000));
    }

    #[test]
    fn test_biometric_simulator() {
        let simulator = BiometricSimulator::new("device1".to_string(), "validator-123");
        let readings = simulator.collect_readings().unwrap();

        assert_eq!(readings.len(), 3);
        assert!(readings.iter().all(|r| r.quality > 0.0 && r.quality <= 1.0));
    }
}
