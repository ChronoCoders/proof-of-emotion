//! Checkpoint system for crash recovery and fast sync

use crate::crypto::{KeyPair, Signature};
use crate::error::{ConsensusError, Result};
use crate::types::Block;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// A checkpoint represents a finalized state at a specific block height
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Checkpoint {
    /// Block height of this checkpoint
    pub height: u64,
    /// Block hash at this checkpoint
    pub block_hash: String,
    /// Epoch number
    pub epoch: u64,
    /// Timestamp when checkpoint was created
    pub timestamp: u64,
    /// Validator signatures (requires ≥67% of stake)
    pub validator_signatures: Vec<ValidatorSignature>,
    /// Total stake that signed this checkpoint
    pub total_stake_signed: u64,
    /// Merkle root of all finalized blocks up to this point
    pub state_root: String,
}

/// A validator's signature on a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidatorSignature {
    /// Validator ID
    pub validator_id: String,
    /// Validator's stake at checkpoint time
    pub stake: u64,
    /// Signature over checkpoint data
    pub signature: Signature,
    /// Validator's public key
    pub public_key: String,
}

/// Manages checkpoint creation and verification
pub struct CheckpointManager {
    /// Stored checkpoints
    checkpoints: Arc<RwLock<Vec<Checkpoint>>>,
    /// Checkpoint interval (create checkpoint every N blocks)
    checkpoint_interval: u64,
    /// Minimum stake percentage required for checkpoint (67%)
    minimum_stake_percentage: u8,
    /// Total stake in the network (for calculating percentages)
    total_network_stake: Arc<RwLock<u64>>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(checkpoint_interval: u64) -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(Vec::new())),
            checkpoint_interval,
            minimum_stake_percentage: 67, // Byzantine threshold
            total_network_stake: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if a checkpoint should be created at this height
    pub fn should_create_checkpoint(&self, height: u64) -> bool {
        height % self.checkpoint_interval == 0
    }

    /// Create a new checkpoint
    pub async fn create_checkpoint(
        &self,
        block: &Block,
        validator_signatures: Vec<ValidatorSignature>,
    ) -> Result<Checkpoint> {
        // Calculate total stake that signed
        let total_stake_signed: u64 = validator_signatures.iter().map(|vs| vs.stake).sum();
        let total_stake = *self.total_network_stake.read().await;

        // Verify we have at least 67% of stake
        if total_stake > 0 {
            let stake_percentage = (total_stake_signed * 100) / total_stake;
            if stake_percentage < self.minimum_stake_percentage as u64 {
                return Err(ConsensusError::config_error(&format!(
                    "Insufficient stake for checkpoint: {}% < {}%",
                    stake_percentage, self.minimum_stake_percentage
                )));
            }
        }

        // Create checkpoint
        let checkpoint = Checkpoint {
            height: block.header.height,
            block_hash: block.hash.clone(),
            epoch: block.header.epoch,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before UNIX_EPOCH")
                .as_millis() as u64,
            validator_signatures,
            total_stake_signed,
            state_root: block.header.merkle_root.clone(),
        };

        // Verify the checkpoint
        self.verify_checkpoint(&checkpoint).await?;

        // Store the checkpoint
        self.checkpoints.write().await.push(checkpoint.clone());

        info!(
            "✅ Checkpoint created at height {} with {}% stake signed",
            checkpoint.height,
            (total_stake_signed * 100) / total_stake.max(1)
        );

        Ok(checkpoint)
    }

    /// Verify a checkpoint's signatures
    pub async fn verify_checkpoint(&self, checkpoint: &Checkpoint) -> Result<bool> {
        if checkpoint.validator_signatures.is_empty() {
            return Err(ConsensusError::signature_verification_failed(
                "No validator signatures in checkpoint",
            ));
        }

        // Create the checkpoint data that was signed
        let checkpoint_data = self.create_checkpoint_data(
            checkpoint.height,
            &checkpoint.block_hash,
            checkpoint.epoch,
            &checkpoint.state_root,
        );

        // Verify each signature
        for validator_sig in &checkpoint.validator_signatures {
            let is_valid = match KeyPair::verify(
                checkpoint_data.as_bytes(),
                &validator_sig.signature,
                &validator_sig.public_key,
            ) {
                Ok(valid) => valid,
                Err(e) => {
                    warn!(
                        "Signature verification error for validator {} in checkpoint: {}",
                        validator_sig.validator_id, e
                    );
                    return Err(e);
                }
            };

            if !is_valid {
                warn!(
                    "Invalid signature from validator {} in checkpoint at height {}",
                    validator_sig.validator_id, checkpoint.height
                );
                return Ok(false);
            }
        }

        // Verify stake percentage
        let total_stake = *self.total_network_stake.read().await;
        if total_stake > 0 {
            let stake_percentage = (checkpoint.total_stake_signed * 100) / total_stake;
            if stake_percentage < self.minimum_stake_percentage as u64 {
                warn!(
                    "Checkpoint at height {} has insufficient stake: {}% < {}%",
                    checkpoint.height, stake_percentage, self.minimum_stake_percentage
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get the latest checkpoint
    pub async fn get_latest_checkpoint(&self) -> Option<Checkpoint> {
        self.checkpoints.read().await.last().cloned()
    }

    /// Get checkpoint at specific height
    pub async fn get_checkpoint_at_height(&self, height: u64) -> Option<Checkpoint> {
        self.checkpoints
            .read()
            .await
            .iter()
            .find(|cp| cp.height == height)
            .cloned()
    }

    /// Get all checkpoints
    pub async fn get_all_checkpoints(&self) -> Vec<Checkpoint> {
        self.checkpoints.read().await.clone()
    }

    /// Update total network stake (for checkpoint verification)
    pub async fn update_total_stake(&self, total_stake: u64) {
        *self.total_network_stake.write().await = total_stake;
    }

    /// Create the canonical checkpoint data for signing
    fn create_checkpoint_data(
        &self,
        height: u64,
        block_hash: &str,
        epoch: u64,
        state_root: &str,
    ) -> String {
        format!(
            "checkpoint:{}:{}:{}:{}",
            height, block_hash, epoch, state_root
        )
    }

    /// Sign a checkpoint (for validators)
    pub fn sign_checkpoint(
        &self,
        height: u64,
        block_hash: &str,
        epoch: u64,
        state_root: &str,
        key_pair: &KeyPair,
    ) -> Result<Signature> {
        let checkpoint_data = self.create_checkpoint_data(height, block_hash, epoch, state_root);
        key_pair
            .sign(checkpoint_data.as_bytes())
            .map_err(|e| ConsensusError::internal(format!("Failed to sign checkpoint: {}", e)))
    }

    /// Prune old checkpoints (keep only last N)
    pub async fn prune_old_checkpoints(&self, keep_count: usize) {
        let mut checkpoints = self.checkpoints.write().await;
        if checkpoints.len() > keep_count {
            let remove_count = checkpoints.len() - keep_count;
            checkpoints.drain(0..remove_count);
            info!("Pruned {} old checkpoints", remove_count);
        }
    }

    /// Get checkpoint statistics
    pub async fn get_checkpoint_statistics(&self) -> CheckpointStatistics {
        let checkpoints = self.checkpoints.read().await;
        let total_checkpoints = checkpoints.len();

        let latest_height = checkpoints.last().map(|cp| cp.height).unwrap_or(0);

        let average_stake_signed = if !checkpoints.is_empty() {
            checkpoints
                .iter()
                .map(|cp| cp.total_stake_signed)
                .sum::<u64>()
                / checkpoints.len() as u64
        } else {
            0
        };

        CheckpointStatistics {
            total_checkpoints,
            latest_checkpoint_height: latest_height,
            checkpoint_interval: self.checkpoint_interval,
            average_stake_signed,
            total_network_stake: *self.total_network_stake.read().await,
        }
    }

    /// Restore state from a checkpoint (returns block hashes to replay)
    pub async fn get_blocks_since_checkpoint(
        &self,
        checkpoint: &Checkpoint,
        current_height: u64,
    ) -> Vec<u64> {
        // Return heights that need to be replayed
        (checkpoint.height + 1..=current_height).collect()
    }
}

/// Statistics about checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStatistics {
    /// Total number of checkpoints
    pub total_checkpoints: usize,
    /// Height of latest checkpoint
    pub latest_checkpoint_height: u64,
    /// Checkpoint creation interval
    pub checkpoint_interval: u64,
    /// Average stake signed across all checkpoints
    pub average_stake_signed: u64,
    /// Total network stake
    pub total_network_stake: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BlockHeader;

    fn create_test_block(height: u64, hash: &str) -> Block {
        Block {
            header: BlockHeader {
                height,
                epoch: height / 10,
                previous_hash: "0".repeat(64),
                merkle_root: "merkle_root".to_string(),
                timestamp: 1000000,
                difficulty: 0,
                nonce: 0,
                validator_id: "validator1".to_string(),
                emotional_score: 85,
                consensus_strength: 80,
            },
            hash: hash.to_string(),
            transactions: vec![],
            signature: String::new(),
            proposer_public_key: String::new(),
            emotional_proof: None,
            consensus_metadata: None,
        }
    }

    #[tokio::test]
    async fn test_checkpoint_interval() {
        let manager = CheckpointManager::new(100);

        assert!(manager.should_create_checkpoint(100));
        assert!(manager.should_create_checkpoint(200));
        assert!(!manager.should_create_checkpoint(150));
        assert!(!manager.should_create_checkpoint(99));
    }

    #[tokio::test]
    async fn test_create_checkpoint() {
        let manager = CheckpointManager::new(100);
        manager.update_total_stake(10_000).await;

        let block = create_test_block(100, "hash100");

        // Create validator signature
        let keypair = KeyPair::generate().unwrap();
        let checkpoint_data = manager.create_checkpoint_data(
            100,
            "hash100",
            10,
            "merkle_root",
        );
        let signature = keypair.sign(checkpoint_data.as_bytes()).unwrap();

        let validator_sig = ValidatorSignature {
            validator_id: "validator1".to_string(),
            stake: 7_000, // 70% of total stake
            signature,
            public_key: keypair.public_key_hex(),
        };

        let checkpoint = manager
            .create_checkpoint(&block, vec![validator_sig])
            .await
            .unwrap();

        assert_eq!(checkpoint.height, 100);
        assert_eq!(checkpoint.block_hash, "hash100");
        assert_eq!(checkpoint.total_stake_signed, 7_000);
    }

    #[tokio::test]
    async fn test_verify_checkpoint() {
        let manager = CheckpointManager::new(100);
        manager.update_total_stake(10_000).await;

        let _block = create_test_block(100, "hash100");

        // Create validator signature
        let keypair = KeyPair::generate().unwrap();
        let signature = manager
            .sign_checkpoint(100, "hash100", 10, "merkle_root", &keypair)
            .unwrap();

        let validator_sig = ValidatorSignature {
            validator_id: "validator1".to_string(),
            stake: 7_000,
            signature,
            public_key: keypair.public_key_hex(),
        };

        let checkpoint = Checkpoint {
            height: 100,
            block_hash: "hash100".to_string(),
            epoch: 10,
            timestamp: 1000000,
            validator_signatures: vec![validator_sig],
            total_stake_signed: 7_000,
            state_root: "merkle_root".to_string(),
        };

        let is_valid = manager.verify_checkpoint(&checkpoint).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_insufficient_stake() {
        let manager = CheckpointManager::new(100);
        manager.update_total_stake(10_000).await;

        let block = create_test_block(100, "hash100");

        let keypair = KeyPair::generate().unwrap();
        let signature = manager
            .sign_checkpoint(100, "hash100", 10, "merkle_root", &keypair)
            .unwrap();

        let validator_sig = ValidatorSignature {
            validator_id: "validator1".to_string(),
            stake: 5_000, // Only 50% - not enough
            signature,
            public_key: keypair.public_key_hex(),
        };

        let result = manager.create_checkpoint(&block, vec![validator_sig]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_latest_checkpoint() {
        let manager = CheckpointManager::new(100);
        manager.update_total_stake(10_000).await;

        assert!(manager.get_latest_checkpoint().await.is_none());

        let block1 = create_test_block(100, "hash100");
        let keypair = KeyPair::generate().unwrap();
        let sig = manager
            .sign_checkpoint(100, "hash100", 10, "merkle_root", &keypair)
            .unwrap();

        let validator_sig = ValidatorSignature {
            validator_id: "validator1".to_string(),
            stake: 7_000,
            signature: sig,
            public_key: keypair.public_key_hex(),
        };

        manager
            .create_checkpoint(&block1, vec![validator_sig.clone()])
            .await
            .unwrap();

        let latest = manager.get_latest_checkpoint().await.unwrap();
        assert_eq!(latest.height, 100);
    }

    #[tokio::test]
    async fn test_checkpoint_statistics() {
        let manager = CheckpointManager::new(100);
        manager.update_total_stake(10_000).await;

        let stats = manager.get_checkpoint_statistics().await;
        assert_eq!(stats.total_checkpoints, 0);
        assert_eq!(stats.checkpoint_interval, 100);

        // Create a checkpoint
        let block = create_test_block(100, "hash100");
        let keypair = KeyPair::generate().unwrap();
        let sig = manager
            .sign_checkpoint(100, "hash100", 10, "merkle_root", &keypair)
            .unwrap();

        let validator_sig = ValidatorSignature {
            validator_id: "validator1".to_string(),
            stake: 7_000,
            signature: sig,
            public_key: keypair.public_key_hex(),
        };

        manager
            .create_checkpoint(&block, vec![validator_sig])
            .await
            .unwrap();

        let stats = manager.get_checkpoint_statistics().await;
        assert_eq!(stats.total_checkpoints, 1);
        assert_eq!(stats.latest_checkpoint_height, 100);
    }
}
