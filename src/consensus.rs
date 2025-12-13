//! Main Proof of Emotion consensus engine

use crate::biometric::{BiometricDevice, BiometricSimulator, EmotionalValidator};
use crate::byzantine::ByzantineDetector;
use crate::error::{ConsensusError, Result};
use crate::types::{Block, Transaction, Vote, VotingResult};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
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
    // Existing metrics
    /// Total epochs processed
    pub total_epochs: u64,
    /// Successful epochs
    pub successful_epochs: u64,
    /// Failed epochs
    pub failed_epochs: u64,
    /// Average epoch duration in milliseconds
    pub average_duration_ms: u64,
    /// Average emotional score across all epochs
    pub average_emotional_score: u8,
    /// Total Byzantine failures detected
    pub byzantine_failures: u64,

    // NEW: Detailed metrics
    /// Total blocks rejected during validation
    pub rejected_blocks: u64,
    /// Total votes rejected (invalid signatures, wrong epoch, etc.)
    pub rejected_votes: u64,
    /// Number of rounds that timed out
    pub timeout_rounds: u64,
    /// Epochs failed due to low emotional fitness
    pub emotional_failures: u64,
    /// Detected network partitions (future use)
    pub network_partitions: u64,
    /// Fork detections (conflicting blocks at same height)
    pub fork_detections: u64,

    // NEW: Performance metrics
    /// Total blocks successfully finalized
    pub blocks_finalized: u64,
    /// Total transactions processed across all blocks
    pub transactions_processed: u64,
    /// Average committee size across epochs
    pub average_committee_size: f64,
    /// Average validator participation rate (percentage)
    pub average_participation_rate: f64,

    // NEW: Timing metrics (in milliseconds)
    /// Average time spent in block proposal phase
    pub average_proposal_time_ms: u64,
    /// Average time spent in voting phase
    pub average_voting_time_ms: u64,
    /// Average time spent in finalization phase
    pub average_finalization_time_ms: u64,

    // NEW: Economic metrics
    /// Total rewards distributed to validators (future integration)
    pub total_rewards_distributed: u64,
    /// Total stake slashed from validators
    pub total_stake_slashed: u64,
    /// Number of currently active validators
    pub active_validators: usize,
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
    /// Byzantine fault detector
    byzantine_detector: Arc<ByzantineDetector>,
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
            byzantine_detector: Arc::new(ByzantineDetector::new()),
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
        let stake = validator.get_stake();
        self.validators.insert(id.clone(), Arc::new(validator));

        info!(
            "✅ Validator {} registered with {} POE stake",
            id,
            stake
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

        // Spawn periodic cleanup task to prevent memory leak from expired transactions
        let cleanup_engine = Arc::clone(&self);
        tokio::spawn(async move {
            let mut cleanup_interval = time::interval(Duration::from_secs(60));
            loop {
                cleanup_interval.tick().await;
                if !*cleanup_engine.is_running.read().await {
                    break;
                }
                cleanup_engine.cleanup_transaction_pool().await;
            }
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

    /// Phase 2: Select committee (optimized with BinaryHeap)
    ///
    /// Uses a min-heap to efficiently select the top k validators by combined score.
    /// Complexity: O(n log k) instead of O(n log n) where k = committee_size
    async fn select_committee(
        &self,
        eligible: &[Arc<EmotionalValidator>],
    ) -> Result<Vec<Arc<EmotionalValidator>>> {
        if eligible.len() <= self.config.committee_size {
            return Ok(eligible.to_vec());
        }

        // Helper struct for ordering validators by score in a heap
        struct OrderedValidator {
            score: u64, // Use integer to avoid f64 comparison issues
            validator: Arc<EmotionalValidator>,
        }

        impl PartialEq for OrderedValidator {
            fn eq(&self, other: &Self) -> bool {
                self.score == other.score
            }
        }

        impl Eq for OrderedValidator {}

        impl PartialOrd for OrderedValidator {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for OrderedValidator {
            fn cmp(&self, other: &Self) -> Ordering {
                // Reverse ordering for max-heap behavior (highest scores first)
                other.score.cmp(&self.score)
            }
        }

        // Use a binary heap to maintain top k validators
        let mut heap = BinaryHeap::with_capacity(self.config.committee_size + 1);

        for validator in eligible {
            let score = validator.get_emotional_score() as f64;
            let stake_weight = (validator.get_stake() as f64).sqrt();
            let reputation = validator.get_reputation() as f64 / 100.0;
            let combined_score = score * stake_weight * reputation;

            // Convert to integer score for reliable comparison
            // Scale by 1000 to preserve precision
            let integer_score = (combined_score * 1000.0) as u64;

            heap.push(OrderedValidator {
                score: integer_score,
                validator: Arc::clone(validator),
            });

            // Keep heap size bounded to committee_size
            if heap.len() > self.config.committee_size {
                heap.pop();
            }
        }

        // Extract validators from heap
        let committee: Vec<_> = heap.into_iter().map(|ov| ov.validator).collect();

        // Update committee size metrics
        let mut metrics = self.metrics.write().await;
        let committee_size = committee.len() as f64;
        if metrics.total_epochs == 0 {
            metrics.average_committee_size = committee_size;
        } else {
            metrics.average_committee_size =
                (metrics.average_committee_size * metrics.total_epochs as f64 + committee_size)
                / (metrics.total_epochs + 1) as f64;
        }
        drop(metrics);

        // TODO: Integrate stake locking when EmotionalStaking is added to consensus engine
        // This prevents nothing-at-stake attacks by locking validator stake during consensus
        // Example integration:
        // for validator in &committee {
        //     self.staking.lock_stake(validator.id(), validator.get_stake(), 1)?;
        // }

        Ok(committee)
    }

    /// Phase 3: Propose block
    async fn propose_block(&self, committee: &[Arc<EmotionalValidator>]) -> Result<Block> {
        let primary = committee
            .first()
            .ok_or_else(|| ConsensusError::committee_selection_failed("Empty committee"))?;

        let pending_txs = self.pending_transactions.lock().await;
        let transactions: Vec<_> = pending_txs.iter().take(1000).cloned().collect();
        drop(pending_txs);

        let finalized_blocks = self.finalized_blocks.read().await;
        let last_height = finalized_blocks.len() as u64;
        let previous_hash = finalized_blocks
            .last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));
        drop(finalized_blocks);

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

        // Record proposal for Byzantine detection (double signing detection)
        if let Err(e) = self
            .byzantine_detector
            .record_proposal(primary.id(), block.header.height, &block.hash)
            .await
        {
            error!("🚨 Byzantine behavior detected during proposal: {}", e);
            // Slash the validator for double signing
            self.slash_validator(primary.id(), "Double signing detected")
                .await?;
            return Err(ConsensusError::invalid_block(e));
        }

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
        let mut byzantine_count = 0;

        // Get expected previous hash, height, and epoch for validation
        let finalized_blocks = self.finalized_blocks.read().await;
        let expected_previous_hash = finalized_blocks
            .last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));
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
            vote.reason = reason.clone();

            // Record vote for Byzantine detection (double voting & equivocation detection)
            if let Err(e) = self.byzantine_detector.record_vote(&vote).await {
                warn!("🚨 Byzantine behavior detected during voting: {}", e);
                byzantine_count += 1;

                // Slash the validator for double voting or equivocation
                if let Err(slash_err) = self
                    .slash_validator(validator.id(), "Double voting or equivocation detected")
                    .await
                {
                    error!(
                        "Failed to slash validator {}: {}",
                        validator.id(),
                        slash_err
                    );
                }

                // Skip this vote - don't count Byzantine votes
                continue;
            }

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

        // Update Byzantine failure metrics
        if byzantine_count > 0 {
            let mut metrics = self.metrics.write().await;
            metrics.byzantine_failures += byzantine_count as u64;
        }

        Ok(VotingResult {
            success,
            consensus_strength,
            participant_count,
            byzantine_count,
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
                .map_err(|e| ConsensusError::internal(format!("System time error: {}", e)))?
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

        // Remove finalized AND expired transactions to prevent memory leak
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ConsensusError::internal(format!("System time error: {}", e)))?
            .as_millis() as u64;
        const MAX_TX_AGE: u64 = 5 * 60 * 1000; // 5 minutes

        let initial_count = pending.len();
        pending
            .retain(|tx| !finalized_hashes.contains(&tx.hash) && !tx.is_expired(now, MAX_TX_AGE));
        let removed_count = initial_count - pending.len();

        if removed_count > 0 {
            info!(
                "🧹 Cleaned up {} transactions ({} finalized, {} expired)",
                removed_count,
                finalized_hashes.len(),
                removed_count.saturating_sub(finalized_hashes.len())
            );
        }

        state.pending_transactions = pending.len();

        info!(
            "🎉 Block {} finalized with {} transactions",
            block.header.height,
            block.transactions.len()
        );

        // Update comprehensive metrics
        let mut metrics = self.metrics.write().await;
        metrics.blocks_finalized += 1;
        metrics.transactions_processed += block.transactions.len() as u64;
        metrics.active_validators = self.validators.len();

        // Update average participation rate
        let new_participation = (voting_result.participant_count as f64 / self.validators.len() as f64) * 100.0;
        if metrics.blocks_finalized == 1 {
            metrics.average_participation_rate = new_participation;
        } else {
            metrics.average_participation_rate =
                (metrics.average_participation_rate * (metrics.blocks_finalized - 1) as f64
                 + new_participation) / metrics.blocks_finalized as f64;
        }

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

    /// Cleanup expired transactions from the transaction pool
    ///
    /// This method removes transactions that have exceeded their TTL (5 minutes).
    /// It runs periodically to prevent memory leaks from rejected/invalid transactions.
    async fn cleanup_transaction_pool(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|e| {
                warn!("System time error during cleanup: {}, using max duration", e);
                std::time::Duration::from_secs(u64::MAX)
            })
            .as_millis() as u64;
        const MAX_TX_AGE: u64 = 5 * 60 * 1000; // 5 minutes

        let mut pending = self.pending_transactions.lock().await;
        let initial_count = pending.len();
        pending.retain(|tx| !tx.is_expired(now, MAX_TX_AGE));
        let removed_count = initial_count - pending.len();

        if removed_count > 0 {
            info!(
                "🧹 Periodic cleanup: removed {} expired transactions",
                removed_count
            );

            // Update state
            let mut state = self.state.write().await;
            state.pending_transactions = pending.len();
        }
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

    /// Slash a validator for Byzantine behavior
    ///
    /// This reduces the validator's reputation and logs the offense
    async fn slash_validator(&self, validator_id: &str, reason: &str) -> Result<()> {
        if let Some(validator_ref) = self.validators.get(validator_id) {
            let validator = validator_ref.value();

            // Reduce reputation by 20 points for Byzantine behavior
            validator.adjust_reputation(-20);

            warn!(
                "⚖️  Slashed validator {} (reputation now {}): {}",
                validator_id,
                validator.get_reputation(),
                reason
            );

            Ok(())
        } else {
            Err(ConsensusError::invalid_block(format!(
                "Validator {} not found for slashing",
                validator_id
            )))
        }
    }

    /// Get Byzantine slashing events
    pub async fn get_byzantine_events(&self) -> Vec<crate::staking::SlashingEvent> {
        self.byzantine_detector.get_slashing_events().await
    }

    /// Cleanup old Byzantine detection data
    pub async fn cleanup_byzantine_data(&self) {
        let current_epoch = self.state.read().await.current_epoch;
        // Keep last 100 epochs of data
        self.byzantine_detector.cleanup_old_data(current_epoch, 100);
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
