//! Emotional staking engine with rewards and slashing

use crate::error::{ConsensusError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Validator in the staking system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator ID
    pub id: String,
    /// Address
    pub address: String,
    /// Total stake
    pub stake: u64,
    /// Locked stake (during consensus participation)
    pub locked_stake: u64,
    /// Available stake (can be withdrawn)
    pub available_stake: u64,
    /// Epoch when stake unlocks (for unbonding)
    pub unlock_epoch: Option<u64>,
    /// Emotional score
    pub emotional_score: u8,
    /// Reputation score
    pub reputation: u8,
    /// Is active
    pub is_active: bool,
    /// Commission percentage
    pub commission: u8,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Total rewards earned
    pub total_rewards: u64,
    /// Total penalties applied
    pub total_penalties: u64,
}

/// Stake entry for delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeEntry {
    /// Validator ID
    pub validator_id: String,
    /// Delegator address
    pub delegator: String,
    /// Staked amount
    pub amount: u64,
    /// Stake timestamp
    pub timestamp: u64,
    /// Lockup period (seconds)
    pub lockup_period: u64,
    /// Accumulated rewards
    pub rewards: u64,
    /// Status
    pub status: StakeStatus,
}

/// Status of a stake
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StakeStatus {
    /// Active
    Active,
    /// Unbonding
    Unbonding,
    /// Slashed
    Slashed,
    /// Withdrawn
    Withdrawn,
}

/// Slashing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    /// Event ID
    pub id: String,
    /// Validator ID
    pub validator_id: String,
    /// Offense type
    pub offense: SlashingOffense,
    /// Severity
    pub severity: SlashingSeverity,
    /// Slashing rate (percentage)
    pub slashing_rate: f64,
    /// Amount slashed
    pub amount: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Evidence
    pub evidence: String,
}

/// Type of slashing offense
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingOffense {
    /// Poor emotional behavior
    PoorEmotionalBehavior,
    /// Missed consensus participation
    MissedConsensus,
    /// Invalid biometric data
    InvalidBiometric,
    /// Double signing
    DoubleSigning,
    /// Extended downtime
    Downtime,
}

/// Severity of slashing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingSeverity {
    /// Minor offense (1% slash)
    Minor,
    /// Major offense (5% slash)
    Major,
    /// Critical offense (15% slash)
    Critical,
}

/// Reward distribution for an epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    /// Epoch number
    pub epoch: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Total rewards distributed
    pub total_rewards: u64,
    /// Rewards by validator
    pub validator_rewards: HashMap<String, u64>,
    /// Rewards by delegator
    pub delegator_rewards: HashMap<String, u64>,
}

/// Emotional staking engine
pub struct EmotionalStaking {
    /// Registered validators
    validators: Arc<RwLock<HashMap<String, Validator>>>,
    /// Active stakes
    stakes: Arc<RwLock<HashMap<String, StakeEntry>>>,
    /// Slashing events
    slashing_events: Arc<RwLock<Vec<SlashingEvent>>>,
    /// Reward history
    reward_history: Arc<RwLock<Vec<RewardDistribution>>>,
    /// Minimum stake
    min_stake: u64,
    /// Current epoch
    current_epoch: Arc<RwLock<u64>>,
}

impl EmotionalStaking {
    /// Create a new staking engine
    pub fn new(min_stake: u64) -> Self {
        Self {
            validators: Arc::new(RwLock::new(HashMap::new())),
            stakes: Arc::new(RwLock::new(HashMap::new())),
            slashing_events: Arc::new(RwLock::new(Vec::new())),
            reward_history: Arc::new(RwLock::new(Vec::new())),
            min_stake,
            current_epoch: Arc::new(RwLock::new(0)),
        }
    }

    /// Register a validator
    pub fn register_validator(
        &self,
        id: String,
        address: String,
        initial_stake: u64,
        commission: u8,
    ) -> Result<()> {
        if initial_stake < self.min_stake {
            return Err(ConsensusError::insufficient_stake(
                initial_stake,
                self.min_stake,
            ));
        }

        if commission > 20 {
            return Err(ConsensusError::config_error("Commission must be <= 20%"));
        }

        let validator = Validator {
            id: id.clone(),
            address,
            stake: initial_stake,
            locked_stake: 0,
            available_stake: initial_stake,
            unlock_epoch: None,
            emotional_score: 0,
            reputation: 100,
            is_active: true,
            commission,
            last_activity: Self::current_timestamp(),
            total_rewards: 0,
            total_penalties: 0,
        };

        self.validators.write().insert(id, validator);
        Ok(())
    }

    /// Delegate stake to a validator
    pub fn delegate_stake(
        &self,
        validator_id: String,
        delegator: String,
        amount: u64,
        lockup_period: u64,
    ) -> Result<()> {
        if amount < 1000 {
            return Err(ConsensusError::insufficient_stake(amount, 1000));
        }

        let validators = self.validators.read();
        let validator = validators
            .get(&validator_id)
            .ok_or_else(|| ConsensusError::validator_not_found(&validator_id))?;

        if !validator.is_active {
            return Err(ConsensusError::invalid_vote("Validator is not active"));
        }
        drop(validators);

        let stake_id = format!("{}:{}", validator_id, delegator);
        let stake = StakeEntry {
            validator_id,
            delegator,
            amount,
            timestamp: Self::current_timestamp(),
            lockup_period,
            rewards: 0,
            status: StakeStatus::Active,
        };

        self.stakes.write().insert(stake_id, stake);
        Ok(())
    }

    /// Apply slashing to a validator
    pub fn slash_validator(
        &self,
        validator_id: &str,
        offense: SlashingOffense,
        evidence: String,
    ) -> Result<()> {
        let mut validators = self.validators.write();
        let validator = validators
            .get_mut(validator_id)
            .ok_or_else(|| ConsensusError::validator_not_found(validator_id))?;

        let severity = Self::determine_severity(offense, &evidence);
        let slashing_rate = match severity {
            SlashingSeverity::Minor => 0.01,
            SlashingSeverity::Major => 0.05,
            SlashingSeverity::Critical => 0.15,
        };

        let slash_amount = (validator.stake as f64 * slashing_rate) as u64;
        validator.stake = validator.stake.saturating_sub(slash_amount);
        validator.total_penalties += slash_amount;

        let reputation_penalty = match severity {
            SlashingSeverity::Minor => 5,
            SlashingSeverity::Major => 10,
            SlashingSeverity::Critical => 20,
        };
        validator.reputation = validator.reputation.saturating_sub(reputation_penalty);

        if validator.stake < self.min_stake {
            validator.is_active = false;
        }

        drop(validators);

        let event = SlashingEvent {
            id: uuid::Uuid::new_v4().as_string(),
            validator_id: validator_id.to_string(),
            offense,
            severity,
            slashing_rate,
            amount: slash_amount,
            timestamp: Self::current_timestamp(),
            evidence,
        };

        self.slashing_events.write().push(event);

        Ok(())
    }

    /// Distribute rewards for an epoch
    pub fn distribute_rewards(
        &self,
        validator_scores: HashMap<String, u8>,
    ) -> Result<RewardDistribution> {
        let epoch = {
            let mut current = self.current_epoch.write();
            *current += 1;
            *current
        };

        let base_reward_pool = 100_000;
        let mut validator_rewards = HashMap::new();
        let mut delegator_rewards = HashMap::new();

        let validators = self.validators.read();
        let total_stake_weight: f64 = validators
            .values()
            .filter(|v| v.is_active)
            .map(|v| (v.stake as f64).sqrt())
            .sum();

        for (validator_id, emotional_score) in validator_scores {
            if let Some(validator) = validators.get(&validator_id) {
                if !validator.is_active {
                    continue;
                }

                let stake_weight = (validator.stake as f64).sqrt();
                let base_reward =
                    ((stake_weight / total_stake_weight) * base_reward_pool as f64) as u64;

                let emotional_multiplier = if emotional_score >= 75 {
                    1.0 + ((emotional_score - 75) as f64 / 100.0) * 0.3
                } else {
                    1.0 - ((75 - emotional_score) as f64 / 100.0) * 0.5
                };

                let total_reward = (base_reward as f64 * emotional_multiplier) as u64;

                let commission_amount = (total_reward * validator.commission as u64) / 100;
                validator_rewards.insert(validator_id.clone(), commission_amount);

                let delegator_reward = total_reward - commission_amount;
                delegator_rewards.insert(validator_id, delegator_reward);
            }
        }
        drop(validators);

        let distribution = RewardDistribution {
            epoch,
            timestamp: Self::current_timestamp(),
            total_rewards: base_reward_pool,
            validator_rewards,
            delegator_rewards,
        };

        self.reward_history.write().push(distribution.clone());

        Ok(distribution)
    }

    /// Determine slashing severity based on offense and evidence
    fn determine_severity(offense: SlashingOffense, _evidence: &str) -> SlashingSeverity {
        match offense {
            SlashingOffense::PoorEmotionalBehavior => SlashingSeverity::Minor,
            SlashingOffense::MissedConsensus => SlashingSeverity::Minor,
            SlashingOffense::InvalidBiometric => SlashingSeverity::Major,
            SlashingOffense::DoubleSigning => SlashingSeverity::Critical,
            SlashingOffense::Downtime => SlashingSeverity::Minor,
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before UNIX_EPOCH - clock may be misconfigured")
            .as_millis() as u64
    }

    /// Get validator
    pub fn get_validator(&self, id: &str) -> Option<Validator> {
        self.validators.read().get(id).cloned()
    }

    /// Get all validators
    pub fn get_all_validators(&self) -> Vec<Validator> {
        self.validators.read().values().cloned().collect()
    }

    /// Get slashing events
    pub fn get_slashing_events(&self) -> Vec<SlashingEvent> {
        self.slashing_events.read().clone()
    }

    /// Get reward history
    pub fn get_reward_history(&self) -> Vec<RewardDistribution> {
        self.reward_history.read().clone()
    }

    /// Lock stake for a validator during consensus participation
    ///
    /// This prevents nothing-at-stake attacks by locking stake while
    /// the validator participates in consensus.
    pub fn lock_stake(&self, validator_id: &str, amount: u64, _epochs: u64) -> Result<()> {
        let mut validators = self.validators.write();
        let validator = validators
            .get_mut(validator_id)
            .ok_or_else(|| ConsensusError::validator_not_found(validator_id))?;

        if validator.available_stake < amount {
            return Err(ConsensusError::insufficient_stake(
                validator.available_stake,
                amount,
            ));
        }

        validator.available_stake = validator.available_stake.saturating_sub(amount);
        validator.locked_stake = validator.locked_stake.saturating_add(amount);

        Ok(())
    }

    /// Unlock stake for a validator after consensus participation
    ///
    /// This makes previously locked stake available again.
    pub fn unlock_stake(&self, validator_id: &str) -> Result<()> {
        let mut validators = self.validators.write();
        let validator = validators
            .get_mut(validator_id)
            .ok_or_else(|| ConsensusError::validator_not_found(validator_id))?;

        // Move all locked stake back to available
        validator.available_stake = validator
            .available_stake
            .saturating_add(validator.locked_stake);
        validator.locked_stake = 0;

        Ok(())
    }

    /// Begin unbonding process for a validator
    ///
    /// Initiates the unbonding period. Stake will be locked for UNBONDING_PERIOD_EPOCHS
    /// before it can be withdrawn. This prevents nothing-at-stake attacks.
    pub fn begin_unbonding(&self, validator_id: &str, amount: u64) -> Result<()> {
        let mut validators = self.validators.write();
        let validator = validators
            .get_mut(validator_id)
            .ok_or_else(|| ConsensusError::validator_not_found(validator_id))?;

        // Check if already unbonding
        if validator.unlock_epoch.is_some() {
            return Err(ConsensusError::config_error(
                "Validator is already unbonding",
            ));
        }

        // Check if enough available stake
        if validator.available_stake < amount {
            return Err(ConsensusError::insufficient_stake(
                validator.available_stake,
                amount,
            ));
        }

        // Get current epoch and calculate unlock epoch
        let current_epoch = *self.current_epoch.read();
        let unlock_epoch = current_epoch + crate::UNBONDING_PERIOD_EPOCHS;

        // Start unbonding
        validator.available_stake = validator.available_stake.saturating_sub(amount);
        validator.locked_stake = validator.locked_stake.saturating_add(amount);
        validator.unlock_epoch = Some(unlock_epoch);
        validator.is_active = false; // Deactivate validator during unbonding

        Ok(())
    }

    /// Complete unbonding and withdraw stake
    ///
    /// Can only be called after the unbonding period has elapsed.
    pub fn complete_unbonding(&self, validator_id: &str) -> Result<u64> {
        let mut validators = self.validators.write();
        let validator = validators
            .get_mut(validator_id)
            .ok_or_else(|| ConsensusError::validator_not_found(validator_id))?;

        let current_epoch = *self.current_epoch.read();

        // Check if unbonding
        let unlock_epoch = validator
            .unlock_epoch
            .ok_or_else(|| ConsensusError::config_error("Validator is not unbonding"))?;

        // Check if unbonding period has elapsed
        if current_epoch < unlock_epoch {
            return Err(ConsensusError::config_error(format!(
                "Unbonding period not complete. {} epochs remaining",
                unlock_epoch - current_epoch
            )));
        }

        // Complete unbonding
        let unbonded_amount = validator.locked_stake;
        validator.stake = validator.stake.saturating_sub(unbonded_amount);
        validator.locked_stake = 0;
        validator.unlock_epoch = None;

        Ok(unbonded_amount)
    }
}

mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
        pub fn as_string(&self) -> String {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            format!(
                "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                rng.gen::<u32>(),
                rng.gen::<u16>(),
                rng.gen::<u16>(),
                rng.gen::<u16>(),
                rng.gen::<u64>() & 0xFFFFFFFFFFFF
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_registration() {
        let staking = EmotionalStaking::new(10_000);

        let result =
            staking.register_validator("validator-1".to_string(), "addr1".to_string(), 10_000, 5);

        assert!(result.is_ok());
        assert!(staking.get_validator("validator-1").is_some());
    }

    #[test]
    fn test_insufficient_stake() {
        let staking = EmotionalStaking::new(10_000);

        let result =
            staking.register_validator("validator-1".to_string(), "addr1".to_string(), 5_000, 5);

        assert!(result.is_err());
    }

    #[test]
    fn test_stake_delegation() {
        let staking = EmotionalStaking::new(10_000);

        staking
            .register_validator("validator-1".to_string(), "addr1".to_string(), 10_000, 5)
            .unwrap();

        let result = staking.delegate_stake(
            "validator-1".to_string(),
            "delegator1".to_string(),
            5_000,
            21 * 24 * 60 * 60,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_slashing() {
        let staking = EmotionalStaking::new(10_000);

        staking
            .register_validator("validator-1".to_string(), "addr1".to_string(), 10_000, 5)
            .unwrap();

        let result = staking.slash_validator(
            "validator-1",
            SlashingOffense::PoorEmotionalBehavior,
            "Score below 40".to_string(),
        );

        assert!(result.is_ok());

        let validator = staking.get_validator("validator-1").unwrap();
        assert!(validator.stake < 10_000);
    }
}
