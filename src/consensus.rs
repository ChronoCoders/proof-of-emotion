//! Main Proof of Emotion consensus engine

use crate::biometric::{BiometricDevice, BiometricSimulator, EmotionalValidator};
use crate::error::{ConsensusError, Result};
use crate::types::{Block, Transaction, Vote, VotingResult};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{error, info, warn};

/// Configuration for consensus engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Duration of each epoch in milliseconds
    pub epoch_duration: u64,
    /// Minimum emotional fitness threshold (0-100)
    pub emotional_threshold: u8,
    /// Byzantine fault tolerance threshold (percentage)
    pub byzantine_threshold: u8,
    /// Number of validators in committee
    pub committee_size: usize,
    /// Minimum stake required (in POE tokens)
    pub minimum_stake: u64,
    /// Voting timeout in milliseconds
    pub voting_timeout: u64,
    /// Proposal timeout in milliseconds
    pub proposal_timeout: u64,
    /// Finality timeout in milliseconds
    pub finality_timeout: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            epoch_duration: 30_000,
            emotional_threshold: 75,
            byzantine_threshold: 67,
            committee_size: 21,
            minimum_stake: 10_000,
            voting_timeout: 8_000,
            proposal_timeout: 10_000,
            finality_timeout: 2_000,
        }
    }
}

/// Current state of consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    /// Current epoch number
    pub current_epoch: u64,
    /// Network health (0-100)
    pub network_health: u8,
    /// Current consensus strength (percentage)
    pub consensus_strength: u8,
    /// Average emotional fitness
    pub emotional_fitness: u8,
    /// Participation rate (percentage)
    pub participation_rate: u8,
    /// Last finalized block height
    pub last_finalized_height: u64,
    /// Number of pending transactions
    pub pending_transactions: usize,
}

/// Phase of a consensus round
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundPhase {
    /// Proposing phase
    Propose,
    /// Voting phase
    Vote,
    /// Commit phase
    Commit,
    /// Finalized
    Finalized,
    /// Aborted
    Aborted,
}

/// Consensus round
pub struct ConsensusRound {
    /// Round ID
    pub id: String,
    /// Current phase
    pub phase: RwLock<RoundPhase>,
    /// Proposed block
    pub proposed_block: Option<Block>,
    /// Votes collected
    pub votes: DashMap<String, Vote>,
    /// Round start time
    pub start_time: std::time::Instant,
}

/// Metrics for consensus performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    /// Total epochs processed
    pub total_epochs: u64,
    /// Successful epochs
    pub successful_epochs: u64,
    /// Failed epochs
    pub failed_epochs: u64,
    /// Average epoch duration
    pub average_duration_ms: u64,
    /// Average emotional score
    pub average_emotional_score: u8,
    /// Total Byzantine failures detected
    pub byzantine_failures: u64,
}

/// Main Proof of Emotion consensus engine
pub struct ProofOfEmotionEngine {
    /// Configuration
    pub config: ConsensusConfig,
    /// Registered validators
    validators: Arc<DashMap<String, Arc<EmotionalValidator>>>,
    /// Pending transactions
    pending_transactions: Arc<Mutex<Vec<Transaction>>>,
    /// Current consensus state
    state: Arc<RwLock<ConsensusState>>,
    /// Is engine running
    is_running: Arc<RwLock<bool>>,
    /// Metrics
    metrics: Arc<RwLock<ConsensusMetrics>>,
    /// Finalized blocks
    finalized_blocks: Arc<RwLock<Vec<Block>>>,
    /// Shutdown signal for graceful termination
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl ProofOfEmotionEngine {
    /// Create a new consensus engine
    pub fn new(config: ConsensusConfig) -> Result<Self> {
        if config.emotional_threshold > 100 {
            return Err(ConsensusError::config_error(
                "Emotional threshold must be <= 100",
            ));
        }
        if config.byzantine_threshold < 51 || config.byzantine_threshold > 100 {
            return Err(ConsensusError::config_error(
                "Byzantine threshold must be 51-100",
            ));
        }
        if config.committee_size == 0 {
            return Err(ConsensusError::config_error("Committee size must be > 0"));
        }

        Ok(Self {
            config,
            validators: Arc::new(DashMap::new()),
            pending_transactions: Arc::new(Mutex::new(Vec::new())),
            state: Arc::new(RwLock::new(ConsensusState {
                current_epoch: 0,
                network_health: 100,
                consensus_strength: 0,
                emotional_fitness: 0,
                participation_rate: 0,
                last_finalized_height: 0,
                pending_transactions: 0,
            })),
            is_running: Arc::new(RwLock::new(false)),
            metrics: Arc::new(RwLock::new(ConsensusMetrics::default())),
            finalized_blocks: Arc::new(RwLock::new(Vec::new())),
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        })
    }

    /// Register a validator
    pub async fn register_validator(&self, validator: EmotionalValidator) -> Result<()> {
        if validator.get_stake() < self.config.minimum_stake {
            return Err(ConsensusError::insufficient_stake(
                validator.get_stake(),
                self.config.minimum_stake,
            ));
        }

        let id = validator.id().to_string();
        self.validators.insert(id.clone(), Arc::new(validator));

        info!(
            "✅ Validator {} registered with {} POE stake",
            id,
            self.validators.get(&id).unwrap().get_stake()
        );

        Ok(())
    }

    /// Start consensus engine
    pub async fn start(self: Arc<Self>) -> Result<()> {
        let mut running = self.is_running.write().await;
        if *running {
            return Err(ConsensusError::AlreadyRunning);
        }
        *running = true;
        drop(running);

        info!("🚀 Starting Proof of Emotion consensus engine");
        info!("⚙️  Epoch duration: {}ms", self.config.epoch_duration);
        info!(
            "💓 Emotional threshold: {}%",
            self.config.emotional_threshold
        );
        info!(
            "🛡️  Byzantine threshold: {}%",
            self.config.byzantine_threshold
        );

        let engine = Arc::clone(&self);

        tokio::spawn(async move {
            engine.epoch_loop().await;
        });

        Ok(())
    }

    /// Stop consensus engine
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.is_running.write().await;
        if !*running {
            return Err(ConsensusError::NotRunning);
        }
        *running = false;
        drop(running);

        info!("🛑 Stopping Proof of Emotion consensus engine");

        // Notify shutdown signal to immediately stop epoch loop
        self.shutdown_signal.notify_waiters();

        Ok(())
    }

    /// Main epoch processing loop
    async fn epoch_loop(&self) {
        let mut interval = time::interval(Duration::from_millis(self.config.epoch_duration));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Check if we should continue running
                    if !*self.is_running.read().await {
                        break;
                    }

                    match self.execute_epoch().await {
                        Ok(_) => {
                            let mut metrics = self.metrics.write().await;
                            metrics.successful_epochs += 1;
                        }
                        Err(e) => {
                            error!("❌ Epoch failed: {}", e);
                            let mut metrics = self.metrics.write().await;
                            metrics.failed_epochs += 1;
                        }
                    }
                }
                _ = self.shutdown_signal.notified() => {
                    info!("🛑 Shutdown signal received, stopping epoch loop");
                    break;
                }
            }
        }
    }

    /// Execute a single epoch
    async fn execute_epoch(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        let mut state = self.state.write().await;
        state.current_epoch += 1;
        let epoch = state.current_epoch;
        drop(state);

        info!("⏰ Starting epoch {}", epoch);

        let eligible_validators = self.perform_emotional_assessment().await?;

        if eligible_validators.is_empty() {
            return Err(ConsensusError::committee_selection_failed(
                "No validators meet emotional fitness threshold",
            ));
        }

        info!(
            "💓 {}/{} validators eligible",
            eligible_validators.len(),
            self.validators.len()
        );

        let committee = self.select_committee(&eligible_validators).await?;

        info!("👥 Committee selected: {} validators", committee.len());

        let proposed_block = self.propose_block(&committee).await?;

        info!(
            "📦 Block {} proposed by {}",
            proposed_block.header.height, proposed_block.header.validator_id
        );

        let voting_result = self.execute_voting(&committee, &proposed_block).await?;

        if !voting_result.success {
            warn!("❌ Voting failed: {:?}", voting_result.reason);
            return Err(ConsensusError::invalid_block(
                voting_result
                    .reason
                    .unwrap_or_else(|| "Voting failed".to_string()),
            ));
        }

        info!(
            "✅ Consensus reached: {}% strength",
            voting_result.consensus_strength
        );

        self.finalize_block(proposed_block, voting_result).await?;

        let duration = start_time.elapsed().as_millis() as u64;
        let mut metrics = self.metrics.write().await;
        metrics.total_epochs += 1;
        metrics.average_duration_ms = (metrics.average_duration_ms * (metrics.total_epochs - 1)
            + duration)
            / metrics.total_epochs;

        info!("✨ Epoch {} completed in {}ms", epoch, duration);

        Ok(())
    }

    /// Phase 1: Perform emotional assessment
    async fn perform_emotional_assessment(&self) -> Result<Vec<Arc<EmotionalValidator>>> {
        let mut eligible = Vec::new();

        for validator_ref in self.validators.iter() {
            let validator = validator_ref.value();

            let simulator =
                BiometricSimulator::new(format!("device_{}", validator.id()), validator.id());

            if let Ok(readings) = simulator.collect_readings() {
                if let Ok(()) = validator.update_emotional_state(readings).await {
                    if validator
                        .is_eligible(self.config.emotional_threshold, self.config.minimum_stake)
                    {
                        eligible.push(Arc::clone(validator));
                    }
                }
            }
        }

        Ok(eligible)
    }

    /// Phase 2: Select committee
    async fn select_committee(
        &self,
        eligible: &[Arc<EmotionalValidator>],
    ) -> Result<Vec<Arc<EmotionalValidator>>> {
        if eligible.len() < self.config.committee_size {
            return Ok(eligible.to_vec());
        }

        let mut scored: Vec<_> = eligible
            .iter()
            .map(|v| {
                let score = v.get_emotional_score() as f64;
                let stake_weight = (v.get_stake() as f64).sqrt();
                let reputation = v.get_reputation() as f64 / 100.0;
                let combined_score = score * stake_weight * reputation;
                (Arc::clone(v), combined_score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(scored
            .into_iter()
            .take(self.config.committee_size)
            .map(|(v, _)| v)
            .collect())
    }

    /// Phase 3: Propose block
    async fn propose_block(&self, committee: &[Arc<EmotionalValidator>]) -> Result<Block> {
        let primary = committee
            .first()
            .ok_or_else(|| ConsensusError::committee_selection_failed("Empty committee"))?;

        let pending_txs = self.pending_transactions.lock().await;
        let transactions: Vec<_> = pending_txs.iter().take(1000).cloned().collect();
        drop(pending_txs);

        let last_height = self.finalized_blocks.read().await.len() as u64;
        let previous_hash = if last_height > 0 {
            self.finalized_blocks
                .read()
                .await
                .last()
                .unwrap()
                .hash
                .clone()
        } else {
            "0".repeat(64)
        };

        // Get current epoch for replay attack prevention
        let current_epoch = self.state.read().await.current_epoch;

        let mut block = Block::new(
            last_height + 1,
            current_epoch,
            previous_hash,
            primary.id().to_string(),
            primary.get_emotional_score(),
            transactions,
        );

        // Sign the block with the proposer's key pair
        block
            .sign(&primary.key_pair)
            .map_err(|e| ConsensusError::internal(format!("Failed to sign block: {}", e)))?;

        Ok(block)
    }

    /// Phase 4: Execute voting
    async fn execute_voting(
        &self,
        committee: &[Arc<EmotionalValidator>],
        block: &Block,
    ) -> Result<VotingResult> {
        let mut votes = Vec::new();
        let mut approved_count = 0;
        let mut total_emotional_score = 0u32;

        // Get expected previous hash, height, and epoch for validation
        let finalized_blocks = self.finalized_blocks.read().await;
        let expected_previous_hash = if finalized_blocks.is_empty() {
            "0".repeat(64)
        } else {
            finalized_blocks.last().unwrap().hash.clone()
        };
        let expected_height = finalized_blocks.len() as u64 + 1;
        drop(finalized_blocks);

        let expected_epoch = self.state.read().await.current_epoch;

        for validator in committee {
            // Perform actual block validation (includes epoch check for replay attack prevention)
            let validation_result = validator.validate_block(
                block,
                &expected_previous_hash,
                expected_height,
                expected_epoch,
            );

            let (approved, reason) = match validation_result {
                Ok(()) => (true, None),
                Err(err_msg) => {
                    warn!("Validator {} rejected block: {}", validator.id(), err_msg);
                    (false, Some(err_msg))
                }
            };

            let mut vote = Vote::new(
                validator.id().to_string(),
                block.hash.clone(),
                block.header.epoch,
                0, // round number (single round per epoch)
                validator.get_emotional_score(),
                approved,
            );
            vote.reason = reason;

            if vote.approved {
                approved_count += 1;
            }
            total_emotional_score += validator.get_emotional_score() as u32;
            votes.push(vote);
        }

        let participant_count = votes.len();
        let required_votes = (self.config.committee_size as f64
            * (self.config.byzantine_threshold as f64 / 100.0))
            .ceil() as usize;

        let success = approved_count >= required_votes;
        let consensus_strength = ((approved_count as f64 / committee.len() as f64) * 100.0) as u8;
        let average_emotional_score = (total_emotional_score / participant_count as u32) as u8;

        Ok(VotingResult {
            success,
            consensus_strength,
            participant_count,
            byzantine_count: 0,
            average_emotional_score,
            participants: committee.iter().map(|v| v.id().to_string()).collect(),
            votes,
            reason: if success {
                None
            } else {
                Some("Insufficient votes".to_string())
            },
        })
    }

    /// Phase 5: Finalize block
    async fn finalize_block(&self, mut block: Block, voting_result: VotingResult) -> Result<()> {
        block.consensus_metadata = Some(crate::types::ConsensusMetadata {
            participant_count: voting_result.participant_count,
            consensus_strength: voting_result.consensus_strength,
            emotional_fitness: voting_result.average_emotional_score,
            byzantine_failures: voting_result.byzantine_count,
            finalized_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            participants: voting_result.participants,
        });

        let mut blocks = self.finalized_blocks.write().await;
        blocks.push(block.clone());

        let mut state = self.state.write().await;
        state.last_finalized_height = block.header.height;
        state.consensus_strength = voting_result.consensus_strength;
        state.emotional_fitness = voting_result.average_emotional_score;
        state.participation_rate =
            ((voting_result.participant_count as f64 / self.validators.len() as f64) * 100.0) as u8;

        let mut pending = self.pending_transactions.lock().await;
        let finalized_hashes: std::collections::HashSet<_> = block
            .transactions
            .iter()
            .map(|tx| tx.hash.clone())
            .collect();
        pending.retain(|tx| !finalized_hashes.contains(&tx.hash));
        state.pending_transactions = pending.len();

        info!(
            "🎉 Block {} finalized with {} transactions",
            block.header.height,
            block.transactions.len()
        );

        Ok(())
    }

    /// Submit a transaction
    pub async fn submit_transaction(&self, transaction: Transaction) -> Result<()> {
        let mut pending = self.pending_transactions.lock().await;
        pending.push(transaction);

        let mut state = self.state.write().await;
        state.pending_transactions = pending.len();

        Ok(())
    }

    /// Get current consensus state
    pub async fn get_state(&self) -> ConsensusState {
        self.state.read().await.clone()
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> ConsensusMetrics {
        self.metrics.read().await.clone()
    }

    /// Get validator count
    pub fn get_validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Get finalized blocks
    pub async fn get_finalized_blocks(&self) -> Vec<Block> {
        self.finalized_blocks.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_engine_creation() {
        let config = ConsensusConfig::default();
        let engine = ProofOfEmotionEngine::new(config).unwrap();
        assert_eq!(engine.get_validator_count(), 0);
    }

    #[tokio::test]
    async fn test_validator_registration() {
        let config = ConsensusConfig::default();
        let engine = ProofOfEmotionEngine::new(config).unwrap();

        let validator = EmotionalValidator::new("validator-1", 10_000).unwrap();
        engine.register_validator(validator).await.unwrap();

        assert_eq!(engine.get_validator_count(), 1);
    }

    #[tokio::test]
    async fn test_insufficient_stake_registration() {
        let config = ConsensusConfig::default();
        let engine = ProofOfEmotionEngine::new(config).unwrap();

        let validator = EmotionalValidator::new("validator-1", 5_000).unwrap();
        let result = engine.register_validator(validator).await;

        assert!(result.is_err());
    }
}
